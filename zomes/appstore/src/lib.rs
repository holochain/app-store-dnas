mod errors;
mod types;
mod validation;

use hdi::prelude::*;
use serde::de::{ Deserializer, Error };
use essence::{ EssenceResponse };
pub use hc_crud::{
    Entity, EntryModel, EntityType,
};

pub use types::{
    CommonFields,
    EntityId,

    HolochainResourceLocation,
    DeprecationNotice,
    LocationTriplet,
    WebAddress,

    PublisherEntry,
    AppEntry,
};


pub use errors::{ ErrorKinds, AppError, UserError };
pub type AppResult<T> = Result<T, ErrorKinds>;

#[derive(Debug, Serialize, Deserialize)]
pub struct GetEntityInput {
    pub id: EntityId,
}
#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateEntityInput<T> {
    pub id: Option<EntityId>,
    pub action: ActionHash,
    pub properties: T,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Metadata {
    pub composition: String,
}

pub type Response<T> = EssenceResponse<T, Metadata, ()>;
pub type EntityResponse<T> = Response<Entity<T>>;

pub fn composition<T>(payload: T, composition: &str) -> Response<T> {
    Response::success( payload, Some(Metadata {
	composition: String::from( composition ),
    }) )
}


#[hdk_entry_defs]
#[unit_enum(UnitEntryTypes)]
pub enum EntryTypes {
    #[entry_def]
    Publisher(PublisherEntry),
    #[entry_def]
    App(AppEntry),
}


#[hdk_link_types]
pub enum LinkTypes {
    Agent,

    Publisher,
    App,

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

	    "Anchor" => Ok(LinkTypes::Anchor),

	    value => Err(D::Error::custom(format!("No LinkTypes value matching '{}'", value ))),
	}
    }
}



impl EntryModel<EntryTypes> for PublisherEntry {
    fn name() -> &'static str { "Publisher" }
    fn get_type(&self) -> EntityType {
	EntityType::new( "publisher", "entry" )
    }
    fn to_input(&self) -> EntryTypes {
	EntryTypes::Publisher(self.clone())
    }
}

impl EntryModel<EntryTypes> for AppEntry {
    fn name() -> &'static str { "App" }
    fn get_type(&self) -> EntityType {
	EntityType::new( "app", "entry" )
    }
    fn to_input(&self) -> EntryTypes {
	EntryTypes::App(self.clone())
    }
}


#[macro_export]
macro_rules! catch { // could change to "trap", "snare", or "capture"
    ( $r:expr ) => {
	match $r {
	    Ok(x) => x,
	    Err(e) => {
		let error = match e {
		    appstore::ErrorKinds::AppError(e) => (&e).into(),
		    appstore::ErrorKinds::UserError(e) => (&e).into(),
		    appstore::ErrorKinds::HDKError(e) => (&e).into(),
		    appstore::ErrorKinds::DnaUtilsError(e) => (&e).into(),
		    appstore::ErrorKinds::FailureResponseError(e) => (&e).into(),
		};
		return Ok(appstore::Response::failure( error, None ))
	    },
	}
    };
    ( $r:expr, $e:expr ) => {
	match $r {
	    Ok(x) => x,
	    Err(e) => return Ok(appstore::Response::failure( (&$e).into(), None )),
	}
    };
}
