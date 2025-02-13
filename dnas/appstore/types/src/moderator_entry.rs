use crate::{
    RmpvValue,
};

use hc_coop_content_sdk::{
    group_ref,
};
use std::collections::BTreeMap;
use hdi::prelude::*;


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
    pub metadata: BTreeMap<String, RmpvValue>,
}
group_ref!( ModeratorActionEntry, group_id );
