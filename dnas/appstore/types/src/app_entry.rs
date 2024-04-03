use crate::{
    HRL,
    EntityId,
    RmpvValue,
    CommonFields,
    DeprecationNotice,
};
use std::collections::BTreeMap;
use hdi::prelude::*;


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
    pub apphub_hrl: HRL,
    pub apphub_hrl_hash: EntryHash,
    pub editors: Vec<AgentPubKey>,

    // common fields
    pub published_at: u64,
    pub last_updated: u64,
    pub metadata: BTreeMap<String, RmpvValue>,

    // optional
    pub deprecation: Option<DeprecationNotice>,
}

impl<'a> CommonFields<'a> for AppEntry {
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
