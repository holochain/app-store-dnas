mod app_entry;
mod publisher_entry;
mod group_anchor_entry;
mod moderator_entry;

pub use coop_content_sdk;

pub use app_entry::*;
pub use publisher_entry::*;
pub use group_anchor_entry::*;
pub use moderator_entry::*;

use std::collections::BTreeMap;
use hdi::prelude::*;


pub type EntityId = ActionHash;
pub type RmpvValue = rmpv::Value;


//
// General-use Structs
//
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct HRL {
    pub dna: DnaHash,
    pub target: AnyDhtHash,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DeprecationNotice {
    pub message: String,

    // optional
    pub recommended_alternatives: Option<Vec<ActionHash>>,
}


// Trait for common fields
pub trait CommonFields<'a> {
    fn author(&'a self) -> &'a AgentPubKey;
    fn published_at(&'a self) -> &'a u64;
    fn last_updated(&'a self) -> &'a u64;
    fn metadata(&'a self) -> &'a BTreeMap<String, RmpvValue>;
}
