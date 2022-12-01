use std::collections::BTreeMap;
use hdi::prelude::*;


pub type EntityId = EntryHash;

//
// General-use Structs
//
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct HolochainResourceLocation {
    pub dna: DnaHash,
    // pub entry: EntryHash,
    pub happ: EntryHash,
    pub gui: EntryHash,

    // // optional
    // pub action: Option<ActionHash>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DeprecationNotice {
    pub message: String,

    // optional
    pub recommended_alternatives: Option<Vec<EntryHash>>,
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
    pub icon: EntityId,
    pub editors: Vec<AgentPubKey>,

    // common fields
    pub author: AgentPubKey,
    pub published_at: u64,
    pub last_updated: u64,
    pub metadata: BTreeMap<String, serde_yaml::Value>,

    // optional
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
    pub name: String,
    pub description: String,
    pub icon: EntityId,
    pub publisher: EntityId,
    pub devhub_address: HolochainResourceLocation,
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
