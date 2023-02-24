use std::collections::BTreeMap;
use hdk::prelude::*;
use hc_crud::{
    now, create_entity, get_entity, update_entity,
    Entity,
};
use appstore::{
    LinkTypes,

    AppEntry,

    EntityId,
    GetEntityInput, UpdateEntityInput,
    HolochainResourceLocation,
};
use crate::{
    AppResult,

    ANCHOR_AGENTS,
    ANCHOR_PUBLISHERS,
    ANCHOR_APPS,
};



#[derive(Debug, Deserialize)]
pub struct CreateInput {
    pub name: String,
    pub description: String,
    pub icon: SerializedBytes,
    pub publisher: EntityId,
    pub devhub_address: HolochainResourceLocation,

    // optional
    pub editors: Option<Vec<AgentPubKey>>,

    pub published_at: Option<u64>,
    pub last_updated: Option<u64>,
    pub metadata: Option<BTreeMap<String, serde_yaml::Value>>,
}


pub fn create(mut input: CreateInput) -> AppResult<Entity<AppEntry>> {
    debug!("Creating App: {}", input.name );
    let pubkey = agent_info()?.agent_initial_pubkey;
    let default_now = now()?;
    let default_editors = vec![ pubkey.clone() ];

    if let Some(ref mut editors) = input.editors {
	if !editors.contains( &pubkey ) {
	    editors.splice( 0..0, default_editors.clone() );
	}
    }

    let icon_addr = crate::save_bytes( input.icon.bytes() )?;

    let app = AppEntry {
	name: input.name,
	description: input.description,
	icon: icon_addr,
	publisher: input.publisher.clone(),
	devhub_address: input.devhub_address,

	editors: input.editors
	    .unwrap_or( default_editors ),

	author: pubkey,
	published_at: input.published_at
	    .unwrap_or( default_now ),
	last_updated: input.last_updated
	    .unwrap_or( default_now ),

	metadata: input.metadata
	    .unwrap_or( BTreeMap::new() ),

	deprecation: None,
    };
    let entity = create_entity( &app )?;

    { // Path via Agent's Apps
	for agent in entity.content.editors.iter() {
	    let (_, pathhash ) = hc_utils::path( ANCHOR_AGENTS, vec![
		// hc_utils::agentid()?,
		agent.to_string(),
		ANCHOR_APPS.to_string(),
	    ]);
	    entity.link_from( &pathhash, LinkTypes::App, None )?;
	}
    }
    { // Path via Publisher's Apps
	let (_, pathhash) = hc_utils::path( ANCHOR_PUBLISHERS, vec![
	    input.publisher.to_string(),
	    ANCHOR_APPS.to_string(),
	]);
	entity.link_from( &pathhash, LinkTypes::App, None )?;
    }
    { // Path via All Apps
	let (_, pathhash) = hc_utils::path_base( ANCHOR_APPS );
	entity.link_from( &pathhash, LinkTypes::App, None )?;
    }

    Ok( entity )
}


pub fn get(input: GetEntityInput) -> AppResult<Entity<AppEntry>> {
    debug!("Get app: {}", input.id );
    let entity : Entity<AppEntry> = get_entity( &input.id )?;

    Ok(	entity )
}


#[derive(Debug, Deserialize, Clone)]
pub struct UpdateProperties {
    pub name: Option<String>,
    pub description: Option<String>,
    pub icon: Option<EntryHash>,
    pub devhub_address: Option<HolochainResourceLocation>,
    pub editors: Option<Vec<AgentPubKey>>,
    pub published_at: Option<u64>,
    pub last_updated: Option<u64>,
    pub metadata: Option<BTreeMap<String, serde_yaml::Value>>,
}
pub type UpdateInput = UpdateEntityInput<UpdateProperties>;

pub fn update(input: UpdateInput) -> AppResult<Entity<AppEntry>> {
    debug!("Updating App: {}", input.base );
    let props = input.properties.clone();
    let mut previous : Option<AppEntry> = None;

    let entity = update_entity(
	&input.base,
	|mut current : AppEntry, _| {
	    previous = Some(current.clone());

	    current.name = props.name
		.unwrap_or( current.name );
	    current.description = props.description
		.unwrap_or( current.description );
	    current.devhub_address = props.devhub_address
		.unwrap_or( current.devhub_address );
	    current.icon = props.icon
		.unwrap_or( current.icon );
	    current.published_at = props.published_at
		.unwrap_or( current.published_at );
	    current.last_updated = props.last_updated
		.unwrap_or( current.last_updated );
	    current.metadata = props.metadata
		.unwrap_or( current.metadata );

	    Ok( current )
	})?;

    // let previous = previous.unwrap();

    Ok( entity )
}
