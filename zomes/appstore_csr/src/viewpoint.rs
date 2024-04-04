use crate::{
    coop_content_sdk,
};

pub use hdk_extensions::hdk;
pub use appstore::{
    LinkTypes,
    RmpvValue,

    AppEntry,
    GroupAnchorEntry,
    ModeratorActionEntry,
    appstore_types,
    hc_crud,
    hdi_extensions,
};
use std::collections::BTreeMap;
use hdk::prelude::*;
use hdi_extensions::{
    guest_error,
};
use hdk_extensions::{
    exists,
    must_get,
    follow_evolutions,
};
use hc_crud::{
    EntryModel,
    Entity,
};
use coop_content_sdk::{
    call_local_zome_decode,
    register_content_to_group,
    register_content_update_to_group,
    get_all_group_content_latest,
};



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
    let apps = crate::app::get_all_apps(())?
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
    let apps = crate::app::get_all_apps(())?
        .into_iter()
        .filter(|entity| removed_app_ids.contains( &entity.id ) )
        .collect();

    Ok( apps )
}
