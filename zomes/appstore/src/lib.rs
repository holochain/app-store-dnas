mod validation;

pub use hdi_extensions;
pub use hdi_extensions::hdi;
pub use hc_crud;
pub use appstore_types::*;

use serde::de::{ Deserializer, Error };
use hdi::prelude::*;
use hdi_extensions::{
    guest_error,
    scoped_type_connector,
    ScopedTypeConnector,
};
use hc_crud::{
    entry_model,
};
use mere_memory_types::{
    MemoryEntry,
};



#[hdk_entry_types]
#[unit_enum(EntryTypesUnit)]
pub enum EntryTypes {
    #[entry_type]
    Publisher(PublisherEntry),
    #[entry_type]
    App(AppEntry),
    #[entry_type]
    ModeratorAction(ModeratorActionEntry),
    #[entry_type]
    GroupAnchor(GroupAnchorEntry),
}

scoped_type_connector!(
    EntryTypesUnit::Publisher,
    EntryTypes::Publisher( PublisherEntry )
);
scoped_type_connector!(
    EntryTypesUnit::App,
    EntryTypes::App( AppEntry )
);
scoped_type_connector!(
    EntryTypesUnit::ModeratorAction,
    EntryTypes::ModeratorAction( ModeratorActionEntry )
);
scoped_type_connector!(
    EntryTypesUnit::GroupAnchor,
    EntryTypes::GroupAnchor( GroupAnchorEntry )
);

// Entity implementations
entry_model!( EntryTypes::Publisher( PublisherEntry ) );
entry_model!( EntryTypes::App( AppEntry ) );
entry_model!( EntryTypes::ModeratorAction( ModeratorActionEntry ) );
entry_model!( EntryTypes::GroupAnchor( GroupAnchorEntry ) );


#[hdk_link_types]
pub enum LinkTypes {
    Agent,

    Publisher,
    App,
    ModeratorAction,
    GroupAnchor,

    Anchor,
}

impl<'de> Deserialize<'de> for LinkTypes {
    fn deserialize<D>(deserializer: D) -> Result<LinkTypes, D::Error>
    where
	D: Deserializer<'de>,
    {
	let name : &str = Deserialize::deserialize(deserializer)?;
	match name {
	    "Agent" => Ok(LinkTypes::Agent),

	    "Publisher" => Ok(LinkTypes::Publisher),
	    "App" => Ok(LinkTypes::App),
	    "ModeratorAction" => Ok(LinkTypes::ModeratorAction),
	    "GroupAnchor" => Ok(LinkTypes::GroupAnchor),

	    "Anchor" => Ok(LinkTypes::Anchor),

	    value => Err(D::Error::custom(format!("No LinkTypes value matching '{}'", value ))),
	}
    }
}


const ICON_SIZE_LIMIT : u64 = 204_800;


pub fn validate_common_fields_create<'a,T,C>(
    action: &C, entry: &'a T
) -> ExternResult<()>
where
    T: CommonFields<'a>,
    C: Into<EntryCreationAction> + Clone,
{
    let creation : EntryCreationAction = action.to_owned().into();

    if entry.author() != creation.author() {
        return Err(guest_error!(format!(
            "Entry author does not match Action author: {} != {}",
            entry.author(), creation.author()
        )));
    }

    Ok(())
}


pub fn validate_common_fields_update<'a,T,C>(
    action: &C, entry: &'a T, prev_entry: &'a T
) -> ExternResult<()>
where
    T: CommonFields<'a>,
    C: Into<EntryCreationAction> + Clone,
{
    let creation : EntryCreationAction = action.to_owned().into();

    if prev_entry.author() != creation.author() {
	return Err(guest_error!(format!(
            "Previous entry author does not match Action author: {} != {}",
            prev_entry.author(), creation.author()
        )));
    }
    else if entry.author() != prev_entry.author() {
	return Err(guest_error!(format!(
            "Cannot change app author: {} => {}",
            prev_entry.author(), entry.author()
        )));
    }

    Ok(())
}


pub fn validate_common_publisher_fields(
    entry: &PublisherEntry
) -> ExternResult<()> {
    let memory : MemoryEntry = must_get_entry( entry.icon.to_owned() )?.try_into()?;
    let icon_size = memory.uncompressed_size
        .unwrap_or( memory.memory_size );

    if icon_size > ICON_SIZE_LIMIT {
	return Err(guest_error!(format!(
            "PublisherEntry icon cannot be larger than {}KB ({} bytes)",
            ICON_SIZE_LIMIT/1024, ICON_SIZE_LIMIT
        )));
    }

    Ok(())
}


pub fn validate_common_app_fields(
    entry: &AppEntry
) -> ExternResult<()> {
    let memory : MemoryEntry = must_get_entry( entry.icon.to_owned() )?.try_into()?;
    let icon_size = memory.uncompressed_size
        .unwrap_or( memory.memory_size );

    if icon_size > ICON_SIZE_LIMIT {
	return Err(guest_error!(format!(
            "AppEntry icon cannot be larger than {}KB ({} bytes)",
            ICON_SIZE_LIMIT/1024, ICON_SIZE_LIMIT
        )));
    }

    Ok(())
}
