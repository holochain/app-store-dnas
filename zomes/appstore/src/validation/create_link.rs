use crate::{
    hdi,
    hdi_extensions,
    LinkTypes,

    ALL_APPS_ANCHOR,

    PublisherEntry,
    AppEntry,
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
    _create: CreateLink,
) -> ExternResult<ValidateCallbackResult> {
    match link_type {
        LinkTypes::AgentToGroup => {
            let agent_base = base_address.clone().into_agent_pub_key()
                .ok_or(guest_error!(
                    format!("Any-linkable hash must be an action hash; not '{}'", base_address )
                ))?;

            // Agent base address must be in group contributors
            let group : GroupEntry = must_get_valid_record(
                target_address.must_be_action_hash()?
            )?.try_into()?;

            if !group.is_contributor( &agent_base ) {
                invalid!(format!(
                    "Agent base address ({}) is not in editor list: {:?}",
                    agent_base, group.contributors(),
                ))
            }

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

            // Agent base address must be in publisher editors
            let group : GroupEntry = must_get_valid_record(
                app_entry.group_ref().1
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
