use crate::{
    hdi,
    hdi_extensions,

    validate_icon_field,

    EntryTypes,

    coop_content_sdk,
};

use coop_content_sdk::{
    validate_group_auth,
    GroupEntry, GroupRef,
};
use hdi::prelude::*;
use hdi_extensions::{
    guest_error,
    valid, invalid,
};


pub fn validation(
    app_entry: EntryTypes,
    create: Create
) -> ExternResult<ValidateCallbackResult> {
    match app_entry {
        EntryTypes::Publisher(entry) => {
            // Check that the author field is in the editors list
            let group : GroupEntry = must_get_valid_record(
                entry.group_ref().1
            )?.try_into()?;

            if !group.is_contributor( &create.author ) {
                invalid!(format!(
                    "Entry author ({}) must be in the editors list: {:?}",
                    create.author, group.contributors(),
                ))
            }

            // Check icon size
            if let Some(icon) = entry.icon {
                validate_icon_field( &icon, "PublisherEntry" )?;
            }

            valid!()
        },
        EntryTypes::App(entry) => {
            // Check that the author field is in the editors list
            if !entry.editors.contains( &create.author ) {
                invalid!(format!(
                    "Entry author ({}) must be in the editors list: {:?}",
                    create.author, entry.editors,
                ))
            }

            // Check icon size
            validate_icon_field( &entry.icon, "AppEntry" )?;

            valid!()
        },
        EntryTypes::AppVersion(_entry) => {
            valid!()
        },
        EntryTypes::ModeratorAction(entry) => {
            // Check that the author is a contributor to the claimed group
            validate_group_auth( &entry, create )
                .map_err(|err| guest_error!(err) )?;

            valid!()
        },
        EntryTypes::GroupAnchor(entry) => {
            // Check that the author is a contributor to the claimed group
            validate_group_auth( &entry, create )
                .map_err(|err| guest_error!(err) )?;

            valid!()
        },
        // _ => invalid!(format!("Create validation not implemented for entry type: {:#?}", create.entry_type )),
    }
}
