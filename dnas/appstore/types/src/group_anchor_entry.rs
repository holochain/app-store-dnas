use hc_coop_content_sdk::{
    group_ref,
};
use hdi::prelude::*;


//
// Group Anchor Entry
//
#[hdk_entry_helper]
#[derive(Clone)]
pub struct GroupAnchorEntry {
    pub group_id: ActionHash,
}
group_ref!( GroupAnchorEntry, group_id, group_id );
