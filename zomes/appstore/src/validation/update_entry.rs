use crate::{
    hdi,
    hdi_extensions,

    validate_common_fields_create,
    validate_icon_field,

    EntryTypes,
    PublisherEntry,
    AppEntry,
    AppVersionEntry,

    hc_coop_content_sdk::{
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
    update: Update,
    _original_action_hash: ActionHash,
    original_entry_hash: EntryHash
) -> ExternResult<ValidateCallbackResult> {
    match app_entry {
        EntryTypes::Publisher(entry) => {
            let previous_entry : PublisherEntry = must_get_entry( original_entry_hash )?
                .try_into()?;

            // Check that the editors list did not change
            if previous_entry.editors != entry.editors {
                invalid!(format!(
                    "Cannot update the editors list: {:?} => {:?}",
                    previous_entry.editors, entry.editors,
                ))
            }

            // Check that the entry is not deprecated
            if entry.deprecation.is_some() && previous_entry.deprecation.is_some() {
                invalid!(format!(
                    "Cannot update deprecated entity unless the deprecation is being reversed",
                ))
            }

            // Check author field matches action author
            validate_common_fields_create( &update, &entry )?;

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
            let previous_entry : AppEntry = must_get_entry( original_entry_hash )?
                .try_into()?;

            // Check that the editors list did not change
            if previous_entry.editors != entry.editors {
                invalid!(format!(
                    "Cannot update the editors list: {:?} => {:?}",
                    previous_entry.editors, entry.editors,
                ))
            }

            // Check that the entry is not deprecated
            if entry.deprecation.is_some() && previous_entry.deprecation.is_some() {
                invalid!(format!(
                    "Cannot update deprecated entity unless the deprecation is being reversed",
                ))
            }

            // Check author field matches action author
            validate_common_fields_create( &update, &entry )?;

            // Check that this action author is in the editor list of the previous publisher entry
            if !entry.editors.contains( &update.author ) {
                invalid!(format!(
                    "Update author ({}) must be in the editors list: {:?}",
                    update.author, entry.editors,
                ))
            }

            // Check icon size
            validate_icon_field( &entry.icon, "AppEntry" )?;

            valid!()
        },
        EntryTypes::AppVersion(entry) => {
            let previous_entry : AppVersionEntry = must_get_entry( original_entry_hash )?
                .try_into()?;
            let app_entry : AppEntry = must_get_valid_record(
                previous_entry.for_app.clone()
            )?.try_into()?;

            // Check that this action author is in the editor list of the previous publisher entry
            if !app_entry.editors.contains( &update.author ) {
                invalid!(format!(
                    "Update author ({}) must be in the App's editors list: {:?}",
                    update.author, app_entry.editors,
                ))
            }

            // Check author field matches action author
            validate_common_fields_create( &update, &entry )?;

            // Fields that cannot be changed
            if previous_entry.for_app != entry.for_app {
                invalid!(format!(
                    "App version field 'for_app' cannot be updated: {} => {}",
                    previous_entry.for_app, entry.for_app,
                ))
            }

            if previous_entry.apphub_hrl != entry.apphub_hrl {
                invalid!(format!(
                    "App version field 'apphub_hrl' cannot be updated: {:?} => {:?}",
                    previous_entry.apphub_hrl, entry.apphub_hrl,
                ))
            }

            if previous_entry.apphub_hrl_hash != entry.apphub_hrl_hash {
                invalid!(format!(
                    "App version field 'apphub_hrl_hash' cannot be updated: {} => {}",
                    previous_entry.apphub_hrl_hash, entry.apphub_hrl_hash,
                ))
            }

            if previous_entry.bundle_hashes != entry.bundle_hashes {
                invalid!(format!(
                    "App version field 'bundle_hashes' cannot be updated: {:?} => {:?}",
                    previous_entry.bundle_hashes, entry.bundle_hashes,
                ))
            }

            valid!()
        },
        EntryTypes::ModeratorAction(entry) => {
            // Check author field matches action author
            if entry.author != update.author {
                invalid!(format!(
                    "Entry author does not match Action author: {} != {}",
                    entry.author, update.author
                ));
            }

            // Check that the author is a contributor to the claimed group
            validate_group_auth( &entry, update )
                .map_err(|err| guest_error!(err) )?;

            valid!()
        },
        EntryTypes::GroupAnchor(entry) => {
            // Check that the author is a contributor to the claimed group
            validate_group_auth( &entry, update )
                .map_err(|err| guest_error!(err) )?;

            valid!()
        },
        // _ => invalid!(format!("Update validation not implemented for entry type: {:#?}", update.entry_type )),
    }
}
