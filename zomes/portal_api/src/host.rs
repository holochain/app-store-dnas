use std::collections::BTreeMap;
use rand::seq::SliceRandom;
use hdk::prelude::*;
use holo_hash::DnaHash;
use hc_crud::{
    now, create_entity, get_entity,// update_entity,
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
    pub dna: DnaHash,
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
	    input.dna.to_string(),
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


#[derive(Debug, Deserialize)]
pub struct GetInput {
    pub dna: DnaHash,
    pub zome: String,
    pub function: String,
}

pub fn list (input: GetInput) -> AppResult<Vec<Entity<HostEntry>>> {
    let (_, pathhash ) = hc_utils::path( ANCHOR_HOSTS, vec![
	&input.dna.to_string(),
	&input.zome,
	&input.function,
    ]);
    let mut links = get_links( pathhash, LinkTypes::Host, None )?;

    if links.len() == 0 {
	return Err("There is no Host for this call".to_string())?;
    }

    links.shuffle(&mut rand::thread_rng());

    let host_targets : Vec<AnyLinkableHash> = links.into_iter()
	.map(|link| link.target)
	.collect();

    let mut hosts : Vec<Entity<HostEntry>> = Vec::new();

    for host_addr in host_targets {
	let host : Entity<HostEntry> = get_entity( &host_addr.clone().into() )?;
	hosts.push( host );
    }

    Ok( hosts )
}
