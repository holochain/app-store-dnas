use crate::{
    hdi,
    hdi_extensions,

    validate_icon_field,

    EntryTypes,
    PublisherEntry,
    AppEntry,

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
            // Check that editors group ID matches the publisher's
            let publisher_entry : PublisherEntry = must_get_valid_record(
                entry.publisher.clone()
            )?.try_into()?;
            if publisher_entry.group_ref().0 != entry.group_ref().0 {
                invalid!(format!(
                    "App entry editors group must match the Publisher's editors group: {} != {}",
                    publisher_entry.group_ref().0, entry.group_ref().0,
                ))
            }

            // Check that the author is a contributor to the claimed group
            validate_group_auth( &entry, create )
                .map_err(|err| guest_error!(err) )?;

            // Check icon size
            validate_icon_field( &entry.icon, "AppEntry" )?;

            valid!()
        },
        EntryTypes::AppVersion(entry) => {
            // Check that editors group ID matches the publisher's
            let app_entry : AppEntry = must_get_valid_record(
                entry.for_app.clone()
            )?.try_into()?;
            if app_entry.group_ref().0 != entry.group_ref().0 {
                invalid!(format!(
                    "App Version entry editors group must match the App's editors group: {} != {}",
                    app_entry.group_ref().0, entry.group_ref().0,
                ))
            }

            // Check that the author is a contributor to the claimed group
            validate_group_auth( &entry, create )
                .map_err(|err| guest_error!(err) )?;

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
