use crate::{
    coop_content_sdk,
    GetForAgentInput,
};

pub use hdk_extensions::hdk;
pub use appstore::{
    LinkTypes,
    appstore_types,
    hc_crud,
    hdi_extensions,
};
use hdk::prelude::*;
use hdi_extensions::{
    guest_error,
    trace_origin_root,
    AnyLinkableHashTransformer,
};
use hdk_extensions::{
    UpdateEntryInput,
    must_get,
    follow_evolutions,
};
use hc_crud::{
    Entity,
};
use coop_content_sdk::{
    create_group, update_group,
    GroupEntry,
};



/// Create a group viewpoint
#[hdk_extern]
pub fn create_group(group: GroupEntry) -> ExternResult<Entity<GroupEntry>> {
    debug!("Creating new group entry: {:#?}", group );
    let action_hash = create_group!( group )?;
    let record = must_get( &action_hash )?;
    let group = GroupEntry::try_from( record )?;

    let entity = Entity {
        id: action_hash.clone(),
        address: hash_entry( group.clone() )?,
        action: action_hash,
        ctype: "group".to_string(),
        content: group,
    };

    Ok( entity )
}


/// Get the current group state
#[hdk_extern]
pub fn get_group(id: ActionHash) -> ExternResult<Entity<GroupEntry>> {
    // We cannot use the macro `coop_content_sdk::get_group!( id )?` because we need the 'latest
    // addr' for entity's action field.
    debug!("Get group latest: {:#?}", id );
    let latest_addr = follow_evolutions( &id )?.last().unwrap().to_owned();
    let record = must_get( &latest_addr )?;
    let group = GroupEntry::try_from( &record )?;

    let entity = Entity {
        id: id,
        address: hash_entry( group.clone() )?,
        action: latest_addr,
        ctype: "group".to_string(),
        content: group,
    };

    Ok( entity )
}


/// Update the group state
#[hdk_extern]
pub fn update_group(input: UpdateEntryInput<GroupEntry>) -> ExternResult<Entity<GroupEntry>> {
    debug!("Update group: {:#?}", input );
    let action_hash = update_group!({
        base: input.base,
        entry: input.entry,
    })?;
    let id = trace_origin_root( &action_hash )?.0;

    let record = must_get( &action_hash )?;
    let group = GroupEntry::try_from( record )?;

    let entity = Entity {
        id: id,
        address: hash_entry( group.clone() )?,
        action: action_hash,
        ctype: "group".to_string(),
        content: group,
    };

    Ok( entity )
}


//
// Editor Group CRUD
//
/// Create a group viewpoint
#[hdk_extern]
pub fn create_editors_group(group: GroupEntry) -> ExternResult<Entity<GroupEntry>> {
    debug!("Creating new (editors) group entry: {:#?}", group );
    let action_hash = create_group!( group )?;
    let record = must_get( &action_hash )?;
    let group = GroupEntry::try_from( record )?;

    let entity = Entity {
        id: action_hash.clone(),
        address: hash_entry( group.clone() )?,
        action: action_hash,
        ctype: "group".to_string(),
        content: group,
    };

    for pubkey in entity.content.contributors() {
        entity.link_from( &pubkey, LinkTypes::AgentToGroup, Some("editors".as_bytes().to_vec()) )?;
    }

    Ok( entity )
}


/// Create a group viewpoint
#[hdk_extern]
pub fn get_editors_groups_for_agent(input: GetForAgentInput) -> ExternResult<Vec<Entity<GroupEntry>>> {
    let editors_groups_links = get_links(
        GetLinksInputBuilder::try_new(
            input.for_agent,
            LinkTypes::AgentToGroup,
        )?
            .tag_prefix( LinkTag::new("editors".as_bytes().to_vec()) )
            .build()
    )?;

    Ok(
        editors_groups_links.into_iter()
            .filter_map( |link| {
                let group_id = link.target.must_be_action_hash().ok()?;
                Some( get_group( group_id ).ok()? )
            })
            .collect()
    )
}
