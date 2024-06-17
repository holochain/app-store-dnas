use crate::{
    HRL,
    EntityId,
    RmpvValue,
    CommonFields,
};
use std::collections::BTreeMap;
use hdi::prelude::*;


#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub struct BundleHashes {
    pub hash: String,
    pub ui_hash: String,
    pub happ_hash: String,
}


//
// App Version Entry
//
#[hdk_entry_helper]
#[derive(Clone)]
pub struct AppVersionEntry {
    pub version: String,
    pub for_app: EntityId,
    pub apphub_hrl: HRL,
    pub apphub_hrl_hash: EntryHash,
    pub bundle_hashes: BundleHashes,

    // common fields
    pub author: AgentPubKey,
    pub published_at: u64,
    pub last_updated: u64,
    pub metadata: BTreeMap<String, RmpvValue>,
}

impl<'a> CommonFields<'a> for AppVersionEntry {
    fn author(&'a self) -> &'a AgentPubKey {
	&self.author
    }
    fn published_at(&'a self) -> &'a u64 {
	&self.published_at
    }
    fn last_updated(&'a self) -> &'a u64 {
	&self.last_updated
    }
    fn metadata(&'a self) -> &'a BTreeMap<String, RmpvValue> {
	&self.metadata
    }
}
