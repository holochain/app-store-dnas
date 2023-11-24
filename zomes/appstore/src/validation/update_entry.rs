use crate::{
    hdi,
    hdi_extensions,

    validate_common_fields_update,
    validate_common_publisher_fields,
    validate_common_app_fields,

    EntryTypes,
    PublisherEntry,
    AppEntry,
};

use hdi::prelude::*;
use hdi_extensions::{
    guest_error,
    valid, invalid,
};
use appstore_types::coop_content_sdk::{
    validate_group_auth,
};


pub fn validation(
    app_entry: EntryTypes,
    update: Update,
    _original_action_hash: ActionHash,
    original_entry_hash: EntryHash
) -> ExternResult<ValidateCallbackResult> {
    match app_entry {
        EntryTypes::Publisher(entry) => {
            let previous_entry : PublisherEntry = must_get_entry( original_entry_hash )?
                .try_into()?;

            validate_common_fields_update( &update, &entry, &previous_entry )?;

            if entry.deprecation.is_some() && previous_entry.deprecation.is_some() {
                invalid!(format!(
                        "Cannot update deprecated entity unless the deprecation is being reversed",
                ))
            }

            validate_common_publisher_fields( &entry )?;

            valid!()
        },
        EntryTypes::App(entry) => {
            let previous_entry : AppEntry = must_get_entry( original_entry_hash )?
                .try_into()?;

            validate_common_fields_update( &update, &entry, &previous_entry )?;

            if entry.deprecation.is_some() && previous_entry.deprecation.is_some() {
                invalid!(format!(
                    "Cannot update deprecated entity unless the deprecation is being reversed",
                ))
            }

            validate_common_app_fields( &entry )?;

            valid!()
        },
        EntryTypes::ModeratorAction(entry) => {
            if entry.author != update.author {
                invalid!(format!(
                    "Entry author does not match Action author: {} != {}",
                    entry.author, update.author
                ));
            }

            validate_group_auth( &entry, update )
                .map_err(|err| guest_error!(err) )?;

            valid!()
        },
        _ => invalid!(format!("Update validation not implemented for entry type: {:#?}", update.entry_type )),
    }
}
