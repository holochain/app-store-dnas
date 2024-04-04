pub mod publisher;
pub mod app;
pub mod app_version;

pub use hdk_extensions::hdk;
pub use appstore::{
    LinkTypes,
    ALL_PUBLISHERS_ANCHOR,
    ALL_APPS_ANCHOR,
    appstore_types,
    hc_crud,
    hdi_extensions,
};
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
use appstore_types::*;
use apphub_sdk::{
    AppEntryInput as AppHubAppEntryInput,
    WebAppEntryInput,
    WebAppPackageEntryInput,
    WebAppPackageVersionEntryInput,
    apphub_types::{
        UiEntry,
        AppEntry as AppHubAppEntry,
        WebAppEntry,
        WebAppPackageEntry,
        WebAppPackageVersionEntry,
        mere_memory_types::{
            MemoryEntry,
            MemoryBlockEntry,
        },
    },
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


#[derive(Debug, Serialize, Deserialize)]
pub struct GetForPublisherInput {
    pub for_publisher: EntityId,
}


#[derive(Debug, Serialize, Deserialize)]
pub struct GetForAppInput {
    pub for_app: EntityId,
}


#[hdk_extern]
fn init(_: ()) -> ExternResult<InitCallbackResult> {
    Ok(InitCallbackResult::Pass)
}


/// Get [`AgentInfo`] for this cell
#[hdk_extern]
pub fn whoami(_: ()) -> ExternResult<AgentInfo> {
    agent_info()
}


// Publisher

/// Get all Publishers for a given [`AgentPubKey`]
#[hdk_extern]
pub fn get_publishers_for_agent(input: GetForAgentInput) -> ExternResult<Vec<Entity<PublisherEntry>>> {
    let collection = hc_crud::get_entities(
        &input.for_agent,
        LinkTypes::AgentToPublisher,
        None
    )?;

    Ok( collection )
}

/// Get Publishers that the current cell agent is a member of
#[hdk_extern]
pub fn get_my_publishers(_:()) -> ExternResult<Vec<Entity<PublisherEntry>>> {
    get_publishers_for_agent( GetForAgentInput {
	for_agent: agent_id()?,
    })
}

/// Get all Publishers
#[hdk_extern]
pub fn get_all_publishers(_: ()) -> ExternResult<Vec<Entity<PublisherEntry>>> {
    let collection = hc_crud::get_entities(
        &ALL_PUBLISHERS_ANCHOR.path_entry_hash()?,
        LinkTypes::AllPublishersToPublisher,
        None
    )?;
    let collection = collection.into_iter()
	.filter(|entity : &Entity<PublisherEntry>| {
	    entity.content.deprecation.is_none()
	})
	.collect();

    Ok( collection )
}


// App

/// Get all Apps for a given [`AgentPubKey`]
#[hdk_extern]
pub fn get_apps_for_agent(input: GetForAgentInput) -> ExternResult<Vec<Entity<AppEntry>>> {
    let collection = hc_crud::get_entities(
        &input.for_agent,
        LinkTypes::AgentToApp,
        None
    )?;

    Ok( collection )
}

/// Get Apps that belong to the given Publisher ID
#[hdk_extern]
pub fn get_apps_for_publisher(input: GetForPublisherInput) -> ExternResult<Vec<Entity<AppEntry>>> {
    let collection = hc_crud::get_entities(
        &input.for_publisher,
        LinkTypes::PublisherToApp,
        None
    )?;

    Ok( collection )
}

/// Get Apps that the current cell agent maintains
#[hdk_extern]
pub fn get_my_apps(_:()) -> ExternResult<Vec<Entity<AppEntry>>> {
    get_apps_for_agent( GetForAgentInput {
	for_agent: agent_id()?,
    })
}

/// Get all Apps
#[hdk_extern]
pub fn get_all_apps(_: ()) -> ExternResult<Vec<Entity<AppEntry>>> {
    let collection = hc_crud::get_entities(
        &ALL_APPS_ANCHOR.path_entry_hash()?,
        LinkTypes::AllAppsToApp,
        None
    )?;
    let collection = collection.into_iter()
	.filter(|entity : &Entity<AppEntry>| {
	    entity.content.deprecation.is_none()
	})
	.collect();

    Ok( collection )
}


// App Version

/// Get App Versions that belong to the given App ID
#[hdk_extern]
pub fn get_app_versions_for_app(input: GetForAppInput) -> ExternResult<Vec<Entity<AppVersionEntry>>> {
    let collection = hc_crud::get_entities(
        &input.for_app,
        LinkTypes::AppToAppVersion,
        None
    )?;

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
            LinkTypes::GroupAnchorToModeratorAction,
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


/// Get moderator actions for the given App ID that were created by the members of the given Group
/// ID
#[hdk_extern]
pub fn get_moderator_actions(input: GetModeratorActionsInput) -> ExternResult<Vec<Entity<ModeratorActionEntry>>> {
    let collection = get_moderator_actions_handler(input)?;

    Ok( collection )
}

/// Get the latest moderated state for a given Group ID and App ID
#[hdk_extern]
pub fn get_moderated_state(input: GetModeratorActionsInput) -> ExternResult<Option<Entity<ModeratorActionEntry>>> {
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

/// Update the moderated state for the given App ID from the viewpoint of the given Groupd ID
#[hdk_extern]
pub fn update_moderated_state(input: UpdateModeratorActionInput) -> ExternResult<Entity<ModeratorActionEntry>> {
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
        create_link(
            group_anchor_hash,
            entity.id.clone(),
            LinkTypes::GroupAnchorToModeratorAction,
            tag.into_bytes()
        )?;

        Ok( entity )
    }
}



//
// Group CRUD
//
/// Create a group viewpoint
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


/// Get the current group state
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


/// Update the group state
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


/// Get all apps from the perspective of the given Group ID
#[hdk_extern]
pub fn viewpoint_get_all_apps(group_id: ActionHash) -> ExternResult<Vec<Entity<AppEntry>>> {
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


/// Get all removed apps from the perspective of the given Group ID
#[hdk_extern]
pub fn viewpoint_get_all_removed_apps(group_id: ActionHash) -> ExternResult<Vec<Entity<AppEntry>>> {
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



/// Calculate the [`EntryHash`] for the AppHub entry type [`WebAppPackageEntry`]
#[hdk_extern]
pub fn hash_webapp_package_entry(input: WebAppPackageEntryInput) -> ExternResult<EntryHash> {
    // debug!("WebAppPackageEntry: {:#?}", input );
    hash_entry( WebAppPackageEntry::from( input ) )
}


/// Calculate the [`EntryHash`] for the AppHub entry type [`WebAppPackageVersionEntry`]
#[hdk_extern]
pub fn hash_webapp_package_version_entry(input: WebAppPackageVersionEntryInput) -> ExternResult<EntryHash> {
    // debug!("WebAppPackageVersionEntry: {:#?}", input );
    hash_entry( WebAppPackageVersionEntry::from( input ) )
}


/// Calculate the [`EntryHash`] for the AppHub entry type [`WebAppEntry`]
#[hdk_extern]
pub fn hash_webapp_entry(input: WebAppEntryInput) -> ExternResult<EntryHash> {
    // debug!("WebAppEntry: {:#?}", input );
    hash_entry( WebAppEntry::from( input ) )
}


/// Calculate the [`EntryHash`] for the AppHub entry type [`UiEntry`]
#[hdk_extern]
pub fn hash_ui_entry(input: UiEntry) -> ExternResult<EntryHash> {
    // debug!("UiEntry: {:#?}", input );
    hash_entry( input )
}


/// Calculate the [`EntryHash`] for the AppHub entry type [`apphub_sdk::apphub_types::AppEntry`]
#[hdk_extern]
pub fn hash_app_entry(input: AppHubAppEntryInput) -> ExternResult<EntryHash> {
    // debug!("AppHubAppEntry: {:#?}", input );
    hash_entry( AppHubAppEntry::from( input ) )
}


/// Calculate the [`EntryHash`] for the AppHub entry type [`MemoryEntry`]
#[hdk_extern]
pub fn hash_mere_memory_entry(input: MemoryEntry) -> ExternResult<EntryHash> {
    // debug!("MemoryEntry: {:#?}", input );
    hash_entry( input )
}


/// Calculate the [`EntryHash`] for the AppHub entry type [`MemoryBlockEntry`]
#[hdk_extern]
pub fn hash_mere_memory_block_entry(input: MemoryBlockEntry) -> ExternResult<EntryHash> {
    // debug!("MemoryBlockEntry: {:#?}", input );
    hash_entry( input )
}
