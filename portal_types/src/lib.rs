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
    pub dna: DnaHash,
    pub capabilities: ZomeCallCapGrant,

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



#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RemoteCallDetails<Z,F,I>
where
    Z: Into<ZomeName>,
    F: Into<FunctionName>,
    I: Serialize + core::fmt::Debug,
{
    pub dna: DnaHash,
    pub zome: Z,
    pub function: F,
    pub payload: I,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct BridgeCallDetails<Z,F,P>
where
    Z: Into<ZomeName>,
    F: Into<FunctionName>,
    P: Serialize + core::fmt::Debug,
{
    pub dna: DnaHash,
    pub zome: Z,
    pub function: F,
    pub payload: P,
}

pub type Payload = rmpv::Value;
pub type RemoteCallInput = RemoteCallDetails<String, String, Payload>;
pub type BridgeCallInput = BridgeCallDetails<String, String, Payload>;
