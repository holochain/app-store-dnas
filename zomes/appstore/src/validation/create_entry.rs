use crate::{
    hdi,
    hdi_extensions,

    validate_common_fields_create,
    validate_common_publisher_fields,
    validate_common_app_fields,

    EntryTypes,
};

use hdi::prelude::*;
use hdi_extensions::{
    valid, invalid,
};


pub fn validation(
    app_entry: EntryTypes,
    create: Create
) -> ExternResult<ValidateCallbackResult> {
    match app_entry {
        EntryTypes::Publisher(entry) => {
            validate_common_fields_create( &create, &entry )?;

            validate_common_publisher_fields( &entry )?;

            valid!()
        },
        EntryTypes::App(entry) => {
            validate_common_fields_create( &create, &entry )?;

            validate_common_app_fields( &entry )?;

            valid!()
        },
        EntryTypes::AppVersion(_entry) => {
            valid!()
        },
        EntryTypes::ModeratorAction(entry) => {
            if entry.author != create.author {
                invalid!(format!(
                    "Entry author does not match Action author: {} != {}",
                    entry.author, create.author
                ))
            }

            valid!()
        },
        EntryTypes::GroupAnchor(_entry) => {
            valid!()
        },
        // _ => invalid!(format!("Create validation not implemented for entry type: {:#?}", create.entry_type )),
    }
}
