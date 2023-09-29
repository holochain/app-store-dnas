pub use coop_content_sdk;

use std::collections::BTreeMap;
use hdi::prelude::*;
use coop_content_sdk::{
    group_ref,
};


pub type EntityId = ActionHash;

//
// General-use Structs
//
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct WebHappConfig {
    pub dna: DnaHash,
    // pub entry: ActionHash,
    pub happ: ActionHash,
    pub gui: Option<ActionHash>,

    // // optional
    // pub action: Option<ActionHash>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DeprecationNotice {
    pub message: String,

    // optional
    pub recommended_alternatives: Option<Vec<ActionHash>>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct LocationTriplet {
    pub country: String,
    pub region: String,
    pub city: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct WebAddress {
    pub url: String,

    // optional
    pub context: Option<String>, // github, gitlab
}


// Trait for common fields
pub trait CommonFields<'a> {
    fn author(&'a self) -> &'a AgentPubKey;
    fn published_at(&'a self) -> &'a u64;
    fn last_updated(&'a self) -> &'a u64;
    fn metadata(&'a self) -> &'a BTreeMap<String, serde_yaml::Value>;
}


//
// Publisher Entry
//
#[hdk_entry_helper]
#[derive(Clone)]
pub struct PublisherEntry {
    pub name: String,
    pub location: LocationTriplet,
    pub website: WebAddress,
    pub icon: EntryHash,
    pub editors: Vec<AgentPubKey>,

    // common fields
    pub author: AgentPubKey,
    pub published_at: u64,
    pub last_updated: u64,
    pub metadata: BTreeMap<String, serde_yaml::Value>,

    // optional
    pub description: Option<String>,
    pub email: Option<String>,
    pub deprecation: Option<DeprecationNotice>,
}

impl<'a> CommonFields<'a> for PublisherEntry {
    fn author(&'a self) -> &'a AgentPubKey {
	&self.author
    }
    fn published_at(&'a self) -> &'a u64 {
	&self.published_at
    }
    fn last_updated(&'a self) -> &'a u64 {
	&self.last_updated
    }
    fn metadata(&'a self) -> &'a BTreeMap<String, serde_yaml::Value> {
	&self.metadata
    }
}



//
// App Entry
//
#[hdk_entry_helper]
#[derive(Clone)]
pub struct AppEntry {
    pub title: String,
    pub subtitle: String,
    pub description: String,
    pub icon: EntryHash,
    pub publisher: EntityId,
    pub devhub_address: WebHappConfig,
    pub editors: Vec<AgentPubKey>,

    // common fields
    pub author: AgentPubKey,
    pub published_at: u64,
    pub last_updated: u64,
    pub metadata: BTreeMap<String, serde_yaml::Value>,

    // optional
    pub deprecation: Option<DeprecationNotice>,
}

impl<'a> CommonFields<'a> for AppEntry {
    fn author(&'a self) -> &'a AgentPubKey {
	&self.author
    }
    fn published_at(&'a self) -> &'a u64 {
	&self.published_at
    }
    fn last_updated(&'a self) -> &'a u64 {
	&self.last_updated
    }
    fn metadata(&'a self) -> &'a BTreeMap<String, serde_yaml::Value> {
	&self.metadata
    }
}



//
// Group Anchor Entry
//
#[hdk_entry_helper]
#[derive(Clone)]
pub struct GroupAnchorEntry {
    pub group_id: ActionHash,
}
group_ref!( GroupAnchorEntry, group_id, group_id );



//
// Moderator Action Entry
//
#[hdk_entry_helper]
#[derive(Clone)]
pub struct ModeratorActionEntry {
    pub group_id: (ActionHash, ActionHash),
    pub author: AgentPubKey,
    pub published_at: u64,
    pub message: String,
    pub subject_id: ActionHash,
    pub metadata: BTreeMap<String, serde_yaml::Value>,
}
group_ref!( ModeratorActionEntry, group_id );
