use std::collections::BTreeMap;
use hdk::prelude::*;
use hc_crud::{
    now, create_entity,// get_entity, update_entity,
    Entity,
};
use portal::{
    LinkTypes,

    HostEntry,
};
use crate::{
    AppResult,

    ANCHOR_HOSTS,
};



#[derive(Debug, Deserialize)]
pub struct CreateInput {
    pub dna: String,
    pub zome: String,
    pub function: String,

    // optional
    pub published_at: Option<u64>,
    pub last_updated: Option<u64>,
    pub metadata: Option<BTreeMap<String, rmpv::Value>>,
}


pub fn create(input: CreateInput) -> AppResult<Entity<HostEntry>> {
    debug!("Creating Host: {}.{}.{}", input.dna, input.zome, input.function );
    let pubkey = agent_info()?.agent_initial_pubkey;
    let default_now = now()?;

    let host = HostEntry {
	dna: input.dna.clone(),
	zome: input.zome.clone().into(),
	function: input.function.clone().into(),

	author: pubkey,
	published_at: input.published_at
	    .unwrap_or( default_now ),
	last_updated: input.last_updated
	    .unwrap_or( default_now ),

	metadata: input.metadata
	    .unwrap_or( BTreeMap::new() ),
    };
    let entity = create_entity( &host )?;

    { // Path via Agent's Hosts
	let (_, pathhash ) = hc_utils::path( ANCHOR_HOSTS, vec![
	    input.dna,
	    input.zome,
	    input.function,
	]);
	entity.link_from( &pathhash, LinkTypes::Host, None )?;
    }

    Ok( entity )
}


// pub fn get(input: GetEntityInput) -> AppResult<Entity<HostEntry>> {
//     debug!("Get host: {}", input.id );
//     let entity : Entity<HostEntry> = get_entity( &input.id )?;

//     Ok(	entity )
// }
