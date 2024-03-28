use crate::{
    RmpvValue,
    CommonFields,
    DeprecationNotice,
};
use std::collections::BTreeMap;
use hdi::prelude::*;


#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct WebAddress {
    pub url: String,

    // optional
    pub context: Option<String>, // github, gitlab
}


//
// Publisher Entry
//
#[hdk_entry_helper]
#[derive(Clone)]
pub struct PublisherEntry {
    pub name: String,
    pub location: String,
    pub website: WebAddress,
    pub editors: Vec<AgentPubKey>,

    // common fields
    pub author: AgentPubKey,
    pub published_at: u64,
    pub last_updated: u64,
    #[serde(default)]
    pub metadata: BTreeMap<String, RmpvValue>,

    // optional
    pub description: Option<String>,
    pub email: Option<String>,
    pub icon: Option<EntryHash>,
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
    fn metadata(&'a self) -> &'a BTreeMap<String, RmpvValue> {
	&self.metadata
    }
}
