mod constants;
mod publisher;
mod app;

pub use hdk_extensions::hdk;
pub use constants::*;
pub use appstore::*;

use std::collections::BTreeMap;
use hdk::prelude::*;
use hdi_extensions::{
    guest_error,
    trace_origin_root,
};
use hdk_extensions::{
    UpdateEntryInput,
    agent_id,
    exists,
    must_get,
    follow_evolutions,
};
use hc_crud::{
    EntryModel,
    Entity,
};
use coop_content_sdk::{
    GroupEntry,
    call_local_zome_decode,
    create_group, update_group,
    register_content_to_group,
    register_content_update_to_group,
    get_all_group_content_latest,
};


#[derive(Debug, Serialize, Deserialize)]
pub struct GetForAgentInput {
    pub for_agent: AgentPubKey,
}


pub fn path_base( base: &str ) -> (Path, EntryHash) {
    path( base, Vec::<String>::new() )
}


pub fn path<T>( base: &str, segments: T ) -> (Path, EntryHash)
where
    T: IntoIterator,
    T::Item: std::fmt::Display,
{
    let mut components : Vec<Component> = vec![];

    for seg in base.split(".") {
	let component = Component::from( format!("{}", seg ).as_bytes().to_vec() );
	components.push( component );
    }

    for seg in segments {
	let component = Component::from( format!("{}", seg ).as_bytes().to_vec() );
	components.push( component );
    }

    let path = Path::from( components );
    let hash = path.path_entry_hash().unwrap();

    ( path, hash )
}


#[hdk_extern]
fn init(_: ()) -> ExternResult<InitCallbackResult> {
    Ok(InitCallbackResult::Pass)
}


#[hdk_extern]
fn whoami(_: ()) -> ExternResult<AgentInfo> {
    agent_info()
}


// Publisher

#[hdk_extern]
fn get_publishers_for_agent(input: GetForAgentInput) -> ExternResult<Vec<Entity<PublisherEntry>>> {
    let (_, pathhash ) = path( ANCHOR_AGENTS, vec![
	input.for_agent.to_string(), ANCHOR_PUBLISHERS.to_string(),
    ]);
    let collection = hc_crud::get_entities( &pathhash, LinkTypes::Publisher, None )?;

    Ok( collection )
}

#[hdk_extern]
fn get_my_publishers(_:()) -> ExternResult<Vec<Entity<PublisherEntry>>> {
    get_publishers_for_agent( GetForAgentInput {
	for_agent: agent_id()?,
    })
}

#[hdk_extern]
fn get_all_publishers(_: ()) -> ExternResult<Vec<Entity<PublisherEntry>>> {
    let (_, pathhash ) = path_base( ANCHOR_PUBLISHERS );
    let collection = hc_crud::get_entities( &pathhash, LinkTypes::Publisher, None )?;
    let collection = collection.into_iter()
	.filter(|entity : &Entity<PublisherEntry>| {
	    entity.content.deprecation.is_none()
	})
	.collect();

    Ok( collection )
}


// App

#[hdk_extern]
fn get_apps_for_agent(input: GetForAgentInput) -> ExternResult<Vec<Entity<AppEntry>>> {
    let (_, pathhash ) = path( ANCHOR_AGENTS, vec![
	input.for_agent.to_string(), ANCHOR_APPS.to_string(),
    ]);
    let collection = hc_crud::get_entities( &pathhash, LinkTypes::App, None )?;

    Ok( collection )
}

#[hdk_extern]
fn get_my_apps(_:()) -> ExternResult<Vec<Entity<AppEntry>>> {
    get_apps_for_agent( GetForAgentInput {
	for_agent: agent_id()?,
    })
}

#[hdk_extern]
fn get_all_apps(_: ()) -> ExternResult<Vec<Entity<AppEntry>>> {
    let (_, pathhash ) = path_base( ANCHOR_APPS );
    let collection = hc_crud::get_entities( &pathhash, LinkTypes::App, None )?;
    let collection = collection.into_iter()
	.filter(|entity : &Entity<AppEntry>| {
	    entity.content.deprecation.is_none()
	})
	.collect();

    Ok( collection )
}


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetModeratorActionsInput {
    pub group_id: ActionHash,
    pub app_id: ActionHash,
}

fn get_moderator_actions_handler(input: GetModeratorActionsInput) -> ExternResult<Vec<Entity<ModeratorActionEntry>>> {
    // - Find group anchor
    // - Find moderator action link with tag 'app::<app_id>'
    // - Follow evolutions for group
    let group_anchor_entry = GroupAnchorEntry {
        group_id: input.group_id.clone(),
    };
    let group_anchor_hash = hash_entry( &group_anchor_entry )?;

    let tag = format!("app::{}", input.app_id );
    let moderator_action_links = get_links(
        GetLinksInputBuilder::try_new(
            group_anchor_hash.clone(),
            LinkTypes::ModeratorAction,
        )?
            .tag_prefix( LinkTag::new(tag) )
            .build()
    )?;

    let mayby_action_history = moderator_action_links
        .iter()
        .min_by_key( |link| link.timestamp );

    Ok( match mayby_action_history {
        Some(link) => {
            type Response = Vec<ActionHash>;
            let history = call_local_zome_decode!(
                Response,
                "coop_content_csr",
                "get_group_content_evolutions",
                coop_content_sdk::GetGroupContentInput {
                    group_id: input.group_id.clone(),
                    content_id: link.target.clone(),
                    full_trace: None,
                }
            )?;

            history
                .into_iter()
                .filter_map(|addr| {
                    let entry = ModeratorActionEntry::try_from( must_get( &addr ).ok()? ).ok()?;
                    Some(
                        Entity {
                            id: addr.clone(),
                            address: hash_entry( entry.clone() ).ok()?,
                            action: addr,
                            ctype: entry.get_type(),
                            content: entry,
                        }
                    )
                })
                .collect()
        },
        None => vec![],
    })
}


#[hdk_extern]
fn get_moderator_actions(input: GetModeratorActionsInput) -> ExternResult<Vec<Entity<ModeratorActionEntry>>> {
    let collection = get_moderator_actions_handler(input)?;

    Ok( collection )
}

#[hdk_extern]
fn get_moderated_state(input: GetModeratorActionsInput) -> ExternResult<Option<Entity<ModeratorActionEntry>>> {
    let history = get_moderator_actions_handler(input)?;
    let state = history.last()
        .map( |state| state.to_owned() );

    Ok( state )
}


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateModeratorActionInput {
    pub group_id: ActionHash,
    pub app_id: ActionHash,
    pub message: String,
    pub metadata: BTreeMap<String, RmpvValue>,
}

#[hdk_extern]
fn update_moderated_state(input: UpdateModeratorActionInput) -> ExternResult<Entity<ModeratorActionEntry>> {
    let actions = get_moderator_actions_handler( GetModeratorActionsInput {
        group_id: input.group_id.clone(),
        app_id: input.app_id.clone(),
    })?;

    let group_rev = follow_evolutions( &input.group_id )?.last().unwrap().to_owned();
    let ma_entry = ModeratorActionEntry {
        group_id: (input.group_id.clone(), group_rev),
        author: agent_id()?,
        published_at: hc_crud::now()?,
        message: input.message,
        subject_id: input.app_id.clone(),
        metadata: input.metadata,
    };

    if actions.len() > 0 {
        let ma_latest = actions.last().unwrap();
        let action_hash = update_entry( ma_latest.action.clone(), ma_entry.clone().to_input() )?;

        register_content_update_to_group!({
            entry: ma_entry.clone(),
            target: action_hash.clone(),
        })?;

        let entity = Entity {
            id: ma_latest.id.clone(),
            address: hash_entry( ma_entry.clone() )?,
            action: action_hash,
            ctype: ma_entry.get_type(),
            content: ma_entry,
        };

        Ok( entity )
    }
    else {
        let entity = hc_crud::create_entity( &ma_entry )?;

        let group_anchor_entry = GroupAnchorEntry {
            group_id: input.group_id,
        };
        let group_anchor_hash = hash_entry( &group_anchor_entry )?;

        if !exists( &group_anchor_hash )? {
            let group_anchor_addr = create_entry( group_anchor_entry.to_input() )?;

            register_content_to_group!({
                entry: group_anchor_entry,
                target: group_anchor_addr,
            })?;
        }

        register_content_to_group!({
            entry: ma_entry,
            target: entity.id.clone(),
        })?;

        let tag = format!("app::{}", input.app_id );
        create_link( group_anchor_hash, entity.id.clone(), LinkTypes::ModeratorAction, tag.into_bytes() )?;

        Ok( entity )
    }
}



//
// Group CRUD
//
#[hdk_extern]
pub fn create_group(group: GroupEntry) -> ExternResult<Entity<GroupEntry>> {
    debug!("Creating new group entry: {:#?}", group );
    let action_hash = create_group!( group )?;
    let record = must_get( &action_hash )?;
    let group = GroupEntry::try_from( record )?;

    let entity = Entity {
        id: action_hash.clone(),
        address: hash_entry( group.clone() )?,
        action: action_hash,
        ctype: "group".to_string(),
        content: group,
    };

    Ok( entity )
}


#[hdk_extern]
pub fn get_group(id: ActionHash) -> ExternResult<Entity<GroupEntry>> {
    debug!("Creating new group entry: {:#?}", id );
    let latest_addr = follow_evolutions( &id )?.last().unwrap().to_owned();
    let record = must_get( &latest_addr )?;
    let group = GroupEntry::try_from( &record )?;

    let entity = Entity {
        id: id,
        address: hash_entry( group.clone() )?,
        action: latest_addr,
        ctype: "group".to_string(),
        content: group,
    };

    Ok( entity )
}


#[hdk_extern]
pub fn update_group(input: UpdateEntryInput<GroupEntry>) -> ExternResult<Entity<GroupEntry>> {
    debug!("Update group: {:#?}", input );
    let action_hash = update_group!({
        base: input.base,
        entry: input.entry,
    })?;
    let id = trace_origin_root( &action_hash )?.0;

    let record = must_get( &action_hash )?;
    let group = GroupEntry::try_from( record )?;

    let entity = Entity {
        id: id,
        address: hash_entry( group.clone() )?,
        action: action_hash,
        ctype: "group".to_string(),
        content: group,
    };

    Ok( entity )
}


#[hdk_extern]
fn viewpoint_get_all_apps(group_id: ActionHash) -> ExternResult<Vec<Entity<AppEntry>>> {
    // - Derive group anchor
    // - Get moderator action links
    // - Get all group content
    let removed_app_ids : Vec<ActionHash> = get_all_group_content_latest!({
        group_id: group_id.clone(),
    })?.into_iter()
        .filter_map(|(_origin, latest)| {
            debug!("Get latest group entry: {}", group_id );
            let addr = latest.into_action_hash()?;
            let record = must_get( &addr ).ok()?;
            Some( ModeratorActionEntry::try_from( record ).ok()? )
        })
        .filter_map(|moderator_action| {
            match moderator_action.metadata.get("remove")? {
                RmpvValue::Boolean(value) => match value {
                    true => Some( moderator_action.subject_id ),
                    false => None,
                },
                _ => None,
            }
        })
        .collect();

    debug!("Removed app IDs from viewpoint {}: {:#?}", group_id, removed_app_ids );
    let apps = get_all_apps(())?
        .into_iter()
        .filter(|entity| !removed_app_ids.contains( &entity.id ) )
        .collect();

    Ok( apps )
}


#[hdk_extern]
fn viewpoint_get_all_removed_apps(group_id: ActionHash) -> ExternResult<Vec<Entity<AppEntry>>> {
    // - Derive group anchor
    // - Get moderator action links
    // - Get all group content
    let removed_app_ids : Vec<ActionHash> = get_all_group_content_latest!({
        group_id: group_id.clone(),
    })?.into_iter()
        .filter_map(|(_origin, latest)| {
            debug!("Get latest group entry: {}", group_id );
            let addr = latest.into_action_hash()?;
            let record = must_get( &addr ).ok()?;
            Some( ModeratorActionEntry::try_from( record ).ok()? )
        })
        .filter_map(|moderator_action| {
            match moderator_action.metadata.get("remove")? {
                RmpvValue::Boolean(value) => match value {
                    true => Some( moderator_action.subject_id ),
                    false => None,
                },
                _ => None,
            }
        })
        .collect();

    debug!("Removed app IDs from viewpoint {}: {:#?}", group_id, removed_app_ids );
    let apps = get_all_apps(())?
        .into_iter()
        .filter(|entity| removed_app_ids.contains( &entity.id ) )
        .collect();

    Ok( apps )
}
