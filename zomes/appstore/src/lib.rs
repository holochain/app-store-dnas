// mod errors;
mod validation;

use hdi::prelude::*;
use serde::de::{ Deserializer, Error };
use essence::{ EssenceResponse };
pub use hc_crud::{
    Entity,
    GetEntityInput, UpdateEntityInput,
    entry_model,
};

pub use appstore_types::{
    CommonFields,
    EntityId,

    WebHappConfig,
    DeprecationNotice,
    LocationTriplet,
    WebAddress,

    PublisherEntry,
    AppEntry,
    ModeratorActionEntry,
    GroupAnchorEntry,
};



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
#[unit_enum(EntryTypesUnit)]
pub enum EntryTypes {
    #[entry_def]
    Publisher(PublisherEntry),
    #[entry_def]
    App(AppEntry),
    #[entry_def]
    ModeratorAction(ModeratorActionEntry),
    #[entry_def]
    GroupAnchor(GroupAnchorEntry),
}

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



#[macro_export]
macro_rules! catch { // could change to "trap", "snare", or "capture"
    ( $r:expr ) => {
	match $r {
	    Ok(x) => x,
	    Err(error) => {
		return Ok($crate::Response::failure( (&error).into(), None ))
	    },
	}
    };
    ( $r:expr, $e:expr ) => {
	match $r {
	    Ok(x) => x,
	    Err(e) => return Ok($crate::Response::failure( (&$e).into(), None )),
	}
    };
}
