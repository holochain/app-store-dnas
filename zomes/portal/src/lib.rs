mod errors;
mod validation;

use hdi::prelude::*;
use serde::de::{ Deserializer, Error };
use essence::{ EssenceResponse };
pub use hc_crud::{
    Entity, EntryModel, EntityType,
};

pub use portal_types::{
    CommonFields,
    EntityId,

    HostEntry,

    RemoteCallDetails,
    BridgeCallDetails,
    Payload,
    RemoteCallInput,
    BridgeCallInput,
};


pub use errors::{ ErrorKinds, AppError, UserError };
pub type AppResult<T> = Result<T, ErrorKinds>;


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
    Host(HostEntry),
}


#[hdk_link_types]
pub enum LinkTypes {
    Agent,

    Host,

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

	    "Host" => Ok(LinkTypes::Host),

	    "Anchor" => Ok(LinkTypes::Anchor),

	    value => Err(D::Error::custom(format!("No LinkTypes value matching '{}'", value ))),
	}
    }
}



impl EntryModel<EntryTypes> for HostEntry {
    fn name() -> &'static str { "Host" }
    fn get_type(&self) -> EntityType {
	EntityType::new( "host", "entry" )
    }
    fn to_input(&self) -> EntryTypes {
	EntryTypes::Host(self.clone())
    }
}



#[macro_export]
macro_rules! catch { // could change to "trap", "snare", or "capture"
    ( $r:expr ) => {
	match $r {
	    Ok(x) => x,
	    Err(e) => {
		let error = match e {
		    portal::ErrorKinds::AppError(e) => (&e).into(),
		    portal::ErrorKinds::UserError(e) => (&e).into(),
		    portal::ErrorKinds::HDKError(e) => (&e).into(),
		    portal::ErrorKinds::DnaUtilsError(e) => (&e).into(),
		    portal::ErrorKinds::FailureResponseError(e) => (&e).into(),
		};
		return Ok(portal::Response::failure( error, None ))
	    },
	}
    };
    ( $r:expr, $e:expr ) => {
	match $r {
	    Ok(x) => x,
	    Err(e) => return Ok(portal::Response::failure( (&$e).into(), None )),
	}
    };
}
