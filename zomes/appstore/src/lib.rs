mod validation;

pub use hdi_extensions;
pub use hdi_extensions::hdi;
pub use hc_crud;
pub use appstore_types;
pub use appstore_types::*;

use serde::de::{ Deserializer, Error };
use lazy_static::lazy_static;
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


lazy_static! {
    pub static ref ALL_APPS_ANCHOR : Path = Path::from(vec![
        Component::from( "apps".as_bytes().to_vec() ),
    ]);
}


#[hdk_entry_types]
#[unit_enum(EntryTypesUnit)]
pub enum EntryTypes {
    #[entry_type]
    Publisher(PublisherEntry),
    #[entry_type]
    App(AppEntry),
    #[entry_type]
    AppVersion(AppVersionEntry),
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
    EntryTypesUnit::AppVersion,
    EntryTypes::AppVersion( AppVersionEntry )
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
entry_model!( EntryTypes::AppVersion( AppVersionEntry ) );
entry_model!( EntryTypes::ModeratorAction( ModeratorActionEntry ) );
entry_model!( EntryTypes::GroupAnchor( GroupAnchorEntry ) );


#[hdk_link_types]
pub enum LinkTypes {
    AgentToGroup,

    AgentToPublisher,

    AgentToApp,
    AllAppsToApp,

    GroupAnchorToModeratorAction,
}

impl<'de> Deserialize<'de> for LinkTypes {
    fn deserialize<D>(deserializer: D) -> Result<LinkTypes, D::Error>
    where
	D: Deserializer<'de>,
    {
	let name : &str = Deserialize::deserialize(deserializer)?;
	match name {
	    "AgentToGroup" => Ok(LinkTypes::AgentToGroup),

	    "AgentToPublisher" => Ok(LinkTypes::AgentToPublisher),

	    "AgentToApp" => Ok(LinkTypes::AgentToApp),
	    "AllAppsToApp" => Ok(LinkTypes::AllAppsToApp),

	    "GroupAnchorToModeratorAction" => Ok(LinkTypes::GroupAnchorToModeratorAction),

	    value => Err(D::Error::custom(format!("No LinkTypes value matching '{}'", value ))),
	}
    }
}


const ICON_SIZE_LIMIT : u64 = 204_800;


pub fn validate_icon_field(
    mere_memory_addr: &EntryHash,
    entry_type_name: &str,
) -> ExternResult<()> {
    let memory : MemoryEntry = must_get_entry( mere_memory_addr.to_owned() )?.try_into()?;
    let icon_size = memory.uncompressed_size
        .unwrap_or( memory.memory_size );

    if icon_size > ICON_SIZE_LIMIT {
	return Err(guest_error!(format!(
            "{} icon cannot be larger than {}KB ({} bytes)",
            entry_type_name, ICON_SIZE_LIMIT/1024, ICON_SIZE_LIMIT
        )));
    }

    Ok(())
}
