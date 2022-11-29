use std::collections::BTreeMap;
use hdi::prelude::*;


//
// General-use Structs
//
pub type EntityId = EntryHash;


// Trait for common fields
pub trait CommonFields<'a> {
    fn author(&'a self) -> &'a AgentPubKey;
    fn published_at(&'a self) -> &'a u64;
    fn last_updated(&'a self) -> &'a u64;
    fn metadata(&'a self) -> &'a BTreeMap<String, rmpv::Value>;
}


//
// Host Entry
//
#[hdk_entry_helper]
#[derive(Clone)]
pub struct HostEntry {
    // pub dna: DnaHash,
    pub dna: String,
    pub zome: ZomeName,
    pub function: FunctionName,
    // pub capabilities: ZomeCallCapGrant,

    // common fields
    pub author: AgentPubKey,
    pub published_at: u64,
    pub last_updated: u64,
    pub metadata: BTreeMap<String, rmpv::Value>,
}

impl<'a> CommonFields<'a> for HostEntry {
    fn author(&'a self) -> &'a AgentPubKey {
	&self.author
    }
    fn published_at(&'a self) -> &'a u64 {
	&self.published_at
    }
    fn last_updated(&'a self) -> &'a u64 {
	&self.last_updated
    }
    fn metadata(&'a self) -> &'a BTreeMap<String, rmpv::Value> {
	&self.metadata
    }
}
