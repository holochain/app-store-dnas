use crate::{
    hdi,
    hdi_extensions,

    validate_icon_field,

    EntryTypes,
    PublisherEntry,
    AppEntry,
    AppVersionEntry,

    coop_content_sdk,
};

use coop_content_sdk::{
    validate_group_auth,
    validate_group_ref,
    validate_group_member,
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
            validate_group_ref( &entry, update.clone() )
                .map_err(|err| guest_error!(format!(
                    "Invalid group reference; {}",
                    err
                )) )?;

            // Check that the author field is in the editors list
            validate_group_member( &entry, update.clone() )
                .map_err(|err| guest_error!(format!(
                    "Invalid editor; {}",
                    err
                )) )?;

            // Check that the entry is not deprecated
            if entry.deprecation.is_some() && previous_entry.deprecation.is_some() {
                invalid!(format!(
                    "Cannot update deprecated entity unless the deprecation is being reversed",
                ))
            }

            // Check icon size
            if let Some(icon) = entry.icon.clone() {
                validate_icon_field( &icon, "PublisherEntry" )?;
            }

            valid!()
        },
        EntryTypes::App(entry) => {
            let previous_entry : AppEntry = must_get_entry( original_entry_hash )?
                .try_into()?;

            // Check that the editors list did not change
            validate_group_ref( &entry, update.clone() )
                .map_err(|err| guest_error!(format!(
                    "Invalid group reference; {}",
                    err
                )) )?;

            // Check that the author field is in the editors list
            validate_group_member( &entry, update.clone() )
                .map_err(|err| guest_error!(format!(
                    "Invalid editor; {}",
                    err
                )) )?;

            // Check that the entry is not deprecated
            if entry.deprecation.is_some() && previous_entry.deprecation.is_some() {
                invalid!(format!(
                    "Cannot update deprecated entity unless the deprecation is being reversed",
                ))
            }

            // Check icon size
            validate_icon_field( &entry.icon, "AppEntry" )?;

            valid!()
        },
        EntryTypes::AppVersion(_entry) => {
            let previous_entry : AppVersionEntry = must_get_entry( original_entry_hash )?
                .try_into()?;
            let app_entry : AppEntry = must_get_valid_record(
                previous_entry.for_app
            )?.try_into()?;

            // Check that the editors list did not change
            validate_group_ref( &app_entry, update.clone() )
                .map_err(|err| guest_error!(format!(
                    "Invalid group reference; {}",
                    err
                )) )?;

            // Check that the author field is in the editors list
            validate_group_member( &app_entry, update.clone() )
                .map_err(|err| guest_error!(format!(
                    "Invalid editor; {}",
                    err
                )) )?;

            valid!()
        },
        EntryTypes::ModeratorAction(entry) => {
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
