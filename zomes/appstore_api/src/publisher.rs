use std::collections::BTreeMap;
use hdk::prelude::*;
use hc_crud::{
    now, create_entity, get_entity, update_entity,
    Entity,
};
use appstore::{
    LinkTypes,

    PublisherEntry,

    EntityId,
    GetEntityInput, UpdateEntityInput,
    LocationTriplet,
    WebAddress,
};
use crate::{
    AppResult,

    ANCHOR_AGENTS,
    ANCHOR_PUBLISHERS,
};



#[derive(Debug, Deserialize)]
pub struct CreateInput {
    pub name: String,
    pub location: LocationTriplet,
    pub website: WebAddress,
    pub icon: EntityId,

    // optional
    pub email: Option<String>,
    pub editors: Option<Vec<AgentPubKey>>,

    pub published_at: Option<u64>,
    pub last_updated: Option<u64>,
    pub metadata: Option<BTreeMap<String, serde_yaml::Value>>,
}


pub fn create(mut input: CreateInput) -> AppResult<Entity<PublisherEntry>> {
    debug!("Creating Publisher: {}", input.name );
    let pubkey = agent_info()?.agent_initial_pubkey;
    let default_now = now()?;
    let default_editors = vec![ pubkey.clone() ];

    if let Some(ref mut editors) = input.editors {
	if !editors.contains( &pubkey ) {
	    editors.splice( 0..0, default_editors.clone() );
	}
    }

    let publisher = PublisherEntry {
	name: input.name,
	location: input.location,
	website: input.website,
	icon: input.icon,

	editors: input.editors
	    .unwrap_or( default_editors ),

	author: pubkey,
	published_at: input.published_at
	    .unwrap_or( default_now ),
	last_updated: input.last_updated
	    .unwrap_or( default_now ),

	metadata: input.metadata
	    .unwrap_or( BTreeMap::new() ),

	email: input.email,
	deprecation: None,
    };
    let entity = create_entity( &publisher )?;

    { // Path via Agent's Publishers
	let (_, pathhash ) = hc_utils::path( ANCHOR_AGENTS, vec![
	    hc_utils::agentid()?,
	    ANCHOR_PUBLISHERS.to_string(),
	]);
	entity.link_from( &pathhash, LinkTypes::Publisher, None )?;
    }
    { // Path via All Publishers
	let (_, pathhash) = hc_utils::path( ANCHOR_PUBLISHERS, vec![
	    entity.id.clone(),
	]);
	entity.link_from( &pathhash, LinkTypes::Publisher, None )?;
    }

    Ok( entity )
}


pub fn get(input: GetEntityInput) -> AppResult<Entity<PublisherEntry>> {
    debug!("Get publisher: {}", input.id );
    let entity : Entity<PublisherEntry> = get_entity( &input.id )?;

    Ok(	entity )
}


#[derive(Debug, Deserialize, Clone)]
pub struct UpdateProperties {
    pub name: Option<String>,
    pub location: Option<LocationTriplet>,
    pub website: Option<WebAddress>,
    pub icon: Option<EntityId>,
    pub email: Option<String>,
    pub editors: Option<Vec<AgentPubKey>>,
    pub published_at: Option<u64>,
    pub last_updated: Option<u64>,
    pub metadata: Option<BTreeMap<String, serde_yaml::Value>>,
}
pub type UpdateInput = UpdateEntityInput<UpdateProperties>;

pub fn update(input: UpdateInput) -> AppResult<Entity<PublisherEntry>> {
    debug!("Updating Publisher: {}", input.action );
    let props = input.properties.clone();
    let mut previous : Option<PublisherEntry> = None;

    let entity = update_entity(
	&input.action,
	|mut current : PublisherEntry, _| {
	    previous = Some(current.clone());

	    current.name = props.name
		.unwrap_or( current.name );

	    Ok( current )
	})?;

    // let previous = previous.unwrap();

    Ok( entity )
}
