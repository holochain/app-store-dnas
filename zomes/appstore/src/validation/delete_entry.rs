use crate::{
    hdi,
    hdi_extensions,
    EntryTypesUnit,

    PublisherEntry,
    AppEntry,
    AppVersionEntry,
};

use hdi::prelude::*;
use hdi_extensions::{
    summon_create_action,
    detect_app_entry_unit,
    // Macros
    valid, invalid,
};


pub fn validation(
    original_action_hash: ActionHash,
    _original_entry_hash: EntryHash,
    delete: Delete
) -> ExternResult<ValidateCallbackResult> {
    let create = summon_create_action( &original_action_hash )?;

    // Always allow creator to delete
    if delete.author == create.author {
        valid!()
    }

    match detect_app_entry_unit( &create )? {
        EntryTypesUnit::Publisher => {
            let publisher_entry : PublisherEntry = must_get_valid_record(
                original_action_hash,
            )?.try_into()?;

            // Allow any publisher editor
            if !publisher_entry.editors.contains( &delete.author ) {
                invalid!(format!(
                    "Delete author ({}) is not in editor list: {:?}",
                    delete.author, publisher_entry.editors,
                ))
            }

            valid!()
        },
        EntryTypesUnit::App => {
            let app_entry : AppEntry = must_get_valid_record(
                original_action_hash,
            )?.try_into()?;

            // Allow any app editor
            if !app_entry.editors.contains( &delete.author ) {
                invalid!(format!(
                    "Delete author ({}) is not in editor list: {:?}",
                    delete.author, app_entry.editors,
                ))
            }

            valid!()
        },
        EntryTypesUnit::AppVersion => {
            let app_version_entry : AppVersionEntry = must_get_valid_record(
                original_action_hash,
            )?.try_into()?;
            let app_entry : AppEntry = must_get_valid_record(
                app_version_entry.for_app,
            )?.try_into()?;

            // Allow any app editor
            if !app_entry.editors.contains( &delete.author ) {
                invalid!(format!(
                    "Delete author ({}) is not in editor list: {:?}",
                    delete.author, app_entry.editors,
                ))
            }

            valid!()
        },
        EntryTypesUnit::ModeratorAction => {
            invalid!(format!(
                "Not authorized to delete moderator action created by {}",
                create.author,
            ))
        },
        EntryTypesUnit::GroupAnchor => {
            invalid!(format!(
                "Not authorized to delete group anchor created by {}",
                create.author,
            ))
        },
        // entry_type_unit => invalid!(format!("Delete validation not implemented for entry type: {:?}", entry_type_unit )),
    }
}
