use crate::{
    hdi,
    hdi_extensions,
    LinkTypes,

    ModeratorActionEntry,

    coop_content_sdk,
};

use coop_content_sdk::{
    GroupEntry,
};
use hdi::prelude::*;
use hdi_extensions::{
    AnyLinkableHashTransformer,
    // Macros
    guest_error,
    valid, invalid,
};


pub fn validation(
    original_action_hash: ActionHash,
    _base_address: AnyLinkableHash,
    delete: DeleteLink,
) -> ExternResult<ValidateCallbackResult> {
    let record = must_get_valid_record( original_action_hash )?;
    let create_link = match record.action() {
        Action::CreateLink(action) => action,
        _ => invalid!(format!("Original action hash does not belong to create link action")),
    };
    let link_type = match LinkTypes::from_type( create_link.zome_index, create_link.link_type )? {
        Some(lt) => lt,
        None => invalid!(format!("No match for LinkTypes")),
    };

    // Always allow link creator to delete their link
    if create_link.author == delete.author {
        valid!()
    }

    match link_type {
        LinkTypes::AgentToGroup => {
            let agent_base = create_link.base_address.clone().into_agent_pub_key()
                .ok_or(guest_error!(
                    format!("Any-linkable hash must be an action hash; not '{}'", create_link.base_address )
                ))?;

            // Allow agent to delete any links on their own anchor
            if agent_base == delete.author {
                valid!()
            }

            invalid!(format!(
                "Not authorized to delete link on agent anchor: {}",
                agent_base,
            ))
        },
        LinkTypes::AgentToPublisher => {
            let agent_base = create_link.base_address.clone().into_agent_pub_key()
                .ok_or(guest_error!(
                    format!("Any-linkable hash must be an action hash; not '{}'", create_link.base_address )
                ))?;

            // Allow agent to delete any links on their own anchor
            if agent_base == delete.author {
                valid!()
            }

            invalid!(format!(
                "Not authorized to delete link on agent anchor: {}",
                agent_base,
            ))
        },
        LinkTypes::AgentToApp => {
            let agent_base = create_link.base_address.clone().into_agent_pub_key()
                .ok_or(guest_error!(
                    format!("Any-linkable hash must be an action hash; not '{}'", create_link.base_address )
                ))?;

            // Allow agent to delete any links on their own anchor
            if agent_base == delete.author {
                valid!()
            }

            invalid!(format!(
                "Not authorized to delete link on agent anchor: {}",
                agent_base,
            ))
        },
        LinkTypes::AllAppsToApp => {
            invalid!(format!(
                "Only the link creator ({}) can delete from the ALL_APPS_ANCHOR",
                create_link.author,
            ))
        },
        LinkTypes::GroupAnchorToModeratorAction => {
            let moderator_action_id = create_link.target_address.must_be_action_hash()?;

            let moderator_action_entry : ModeratorActionEntry = must_get_valid_record(
                moderator_action_id
            )?.try_into()?;
            let group_entry : GroupEntry = must_get_valid_record(
                moderator_action_entry.group_id.1.clone()
            )?.try_into()?;

            // Check that the action author is a contributor in the Moderator Action group revision
            if !group_entry.contributors().contains( &delete.author ) {
                invalid!(format!(
                    "Delete author ({}) is not a contributor in Group ({}) revision ({})",
                    delete.author, moderator_action_entry.group_id.0, moderator_action_entry.group_id.1,
                ))
            }

            valid!()
        },
    }
}
