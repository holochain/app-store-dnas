use crate::{
    hdi,
    hdi_extensions,
    LinkTypes,

    ALL_APPS_ANCHOR,

    PublisherEntry,
    AppEntry,
    AppVersionEntry,
    ModeratorActionEntry,
    GroupAnchorEntry,

    coop_content_sdk,
};

use coop_content_sdk::{
    GroupEntry, GroupRef,
};
use hdi::prelude::*;
use hdi_extensions::{
    AnyLinkableHashTransformer,
    verify_app_entry_struct,
    // Macros
    guest_error,
    valid, invalid,
};


pub fn validation(
    base_address: AnyLinkableHash,
    target_address: AnyLinkableHash,
    link_type: LinkTypes,
    _tag: LinkTag,
    create: CreateLink,
) -> ExternResult<ValidateCallbackResult> {
    match link_type {
        LinkTypes::AgentToGroup => {
            valid!()
        },
        LinkTypes::AgentToPublisher => {
            let publisher_entry : PublisherEntry = must_get_valid_record(
                target_address.must_be_action_hash()?
            )?.try_into()?;

            let agent_base = base_address.clone().into_agent_pub_key()
                .ok_or(guest_error!(
                    format!("Any-linkable hash must be an action hash; not '{}'", base_address )
                ))?;

            // Agent base address must be in publisher editors
            let group : GroupEntry = must_get_valid_record(
                publisher_entry.group_ref().1
            )?.try_into()?;

            if !group.is_contributor( &agent_base ) {
                invalid!(format!(
                    "Agent base address ({}) is not in editor list: {:?}",
                    agent_base, group.contributors(),
                ))
            }

            // TODO: somehow verify that the agent wants these links on their anchor

            valid!()
        },
        LinkTypes::AgentToApp => {
            let app_entry : AppEntry = must_get_valid_record(
                target_address.must_be_action_hash()?
            )?.try_into()?;

            let agent_base = base_address.clone().into_agent_pub_key()
                .ok_or(guest_error!(
                    format!("Any-linkable hash must be an action hash; not '{}'", base_address )
                ))?;

            // Agent base address must be in app editors
            if !app_entry.editors.contains( &agent_base ) {
                invalid!(format!(
                    "Agent base address ({}) is not in editor list: {:?}",
                    agent_base, app_entry.editors,
                ))
            }

            // TODO: somehow verify that the agent wants these links on their anchor

            valid!()
        },
        LinkTypes::PublisherToApp => {
            let publisher_id = base_address.must_be_action_hash()?;

            let publisher_entry : PublisherEntry = must_get_valid_record(
                publisher_id
            )?.try_into()?;

            // Author must be in publisher editors
            let group : GroupEntry = must_get_valid_record(
                publisher_entry.group_ref().1
            )?.try_into()?;

            if !group.is_contributor( &create.author ) {
                invalid!(format!(
                    "Link author ({}) is not in editor list: {:?}",
                    create.author, group.contributors(),
                ))
            }

            verify_app_entry_struct::<AppEntry>( &target_address )?;

            valid!()
        },
        LinkTypes::AllAppsToApp => {
            if base_address != ALL_APPS_ANCHOR.path_entry_hash()?.into() {
                invalid!(format!(
                    "Base address must be the ALL_APP_ANCHOR ({})",
                    ALL_APPS_ANCHOR.path_entry_hash()?,
                ))
            }

            verify_app_entry_struct::<AppEntry>( &target_address )?;

            valid!()
        },
        LinkTypes::AppToAppVersion => {
            let app_id = base_address.must_be_action_hash()?;

            let app_entry : AppEntry = must_get_valid_record(
                app_id.clone()
            )?.try_into()?;

            // Link author must be in app editors
            if !app_entry.editors.contains( &create.author ) {
                invalid!(format!(
                    "Author ({}) cannot link app -> app version because they are not in the editor list ({})",
                    create.author, base_address,
                ))
            }

            let app_version_entry : AppVersionEntry = must_get_valid_record(
                target_address.must_be_action_hash()?
            )?.try_into()?;

            // Version must belong to app base address
            if app_version_entry.for_app != app_id {
                invalid!(format!(
                    "App base address does not match the app reference in version entry: {} != {}",
                    app_id, app_version_entry.for_app,
                ))
            }

            valid!()
        },
        LinkTypes::GroupAnchorToModeratorAction => {
            let group_anchor_hash = base_address.must_be_entry_hash()?;
            let moderator_action_id = target_address.must_be_action_hash()?;

            let group_anchor_entry : GroupAnchorEntry = must_get_entry(
                group_anchor_hash
            )?.try_into()?;
            let moderator_action_entry : ModeratorActionEntry = must_get_valid_record(
                moderator_action_id
            )?.try_into()?;

            // Check that the Moderator Action group matches the Group Anchor
            if group_anchor_entry.group_id != moderator_action_entry.group_id.0 {
                invalid!(format!(
                    "Moderator Action does not belong to Group Anchor: {} != {}",
                    group_anchor_entry.group_id, moderator_action_entry.group_id.0,
                ))
            }

            valid!()
        },
        // _ => invalid!(format!("Create link validation not implemented for link type: {:#?}", create.link_type )),
    }
}
