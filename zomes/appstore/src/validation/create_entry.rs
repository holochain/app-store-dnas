use crate::{
    hdi,
    hdi_extensions,

    validate_common_fields_create,
    validate_icon_field,

    EntryTypes,

    coop_content_sdk::{
        validate_group_auth,
    },
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
            // Check author field matches action author
            validate_common_fields_create( &create, &entry )?;

            // Check that the author field is in the editors list
            if !entry.editors.contains( &entry.author ) {
                invalid!(format!(
                    "Entry author ({}) must be in the editors list: {:?}",
                    entry.author, entry.editors,
                ))
            }

            // Check icon size
            if let Some(icon) = entry.icon {
                validate_icon_field( &icon, "PublisherEntry" )?;
            }

            valid!()
        },
        EntryTypes::App(entry) => {
            // Check author field matches action author
            validate_common_fields_create( &create, &entry )?;

            // Check that the author field is in the editors list
            if !entry.editors.contains( &entry.author ) {
                invalid!(format!(
                    "Entry author ({}) must be in the editors list: {:?}",
                    entry.author, entry.editors,
                ))
            }

            // Check icon size
            validate_icon_field( &entry.icon, "AppEntry" )?;

            valid!()
        },
        EntryTypes::AppVersion(entry) => {
            // Check author field matches action author
            validate_common_fields_create( &create, &entry )?;

            valid!()
        },
        EntryTypes::ModeratorAction(entry) => {
            // Check author field matches action author
            if entry.author != create.author {
                invalid!(format!(
                    "Entry author does not match Action author: {} != {}",
                    entry.author, create.author
                ))
            }

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
