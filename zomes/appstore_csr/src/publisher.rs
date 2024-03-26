use crate::{
    hdk,
};

use std::collections::BTreeMap;
use hdk::prelude::*;
use hdk_extensions::{
    agent_id,
};
use appstore::{
    LinkTypes,
    RmpvValue,
    LocationTriplet,
    WebAddress,
    DeprecationNotice,

    ALL_PUBLISHERS_ANCHOR,
    PublisherEntry,

    hc_crud::{
        now, create_entity, get_entity, update_entity,
        Entity,
        GetEntityInput, UpdateEntityInput,
    },
};



#[derive(Debug, Deserialize)]
pub struct CreateInput {
    pub name: String,
    pub location: LocationTriplet,
    pub website: WebAddress,
    pub icon: EntryHash,

    // optional
    pub description: Option<String>,
    pub email: Option<String>,
    pub editors: Option<Vec<AgentPubKey>>,

    pub published_at: Option<u64>,
    pub last_updated: Option<u64>,
    pub metadata: Option<BTreeMap<String, RmpvValue>>,
}


#[hdk_extern]
pub fn create_publisher(mut input: CreateInput) -> ExternResult<Entity<PublisherEntry>> {
    debug!("Creating Publisher: {}", input.name );
    let pubkey = agent_id()?;
    let default_now = now()?;
    let default_editors = vec![ pubkey.clone() ];

    if let Some(ref mut editors) = input.editors {
	if !editors.contains( &pubkey ) {
	    editors.splice( 0..0, default_editors.clone() );
	}
    }

    let publisher = PublisherEntry {
	name: input.name,
	description: input.description,
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
	for agent in entity.content.editors.iter() {
	    entity.link_from(
                agent,
                LinkTypes::AgentToPublisher,
                None
            )?;
	}
    }
    { // Path via All Publishers
	entity.link_from(
            &ALL_PUBLISHERS_ANCHOR.path_entry_hash()?,
            LinkTypes::AllPublishersToPublisher,
            None
        )?;
    }

    Ok( entity )
}


#[hdk_extern]
pub fn get_publisher(input: GetEntityInput) -> ExternResult<Entity<PublisherEntry>> {
    debug!("Get publisher: {}", input.id );
    let entity : Entity<PublisherEntry> = get_entity( &input.id )?;

    Ok(	entity )
}


#[derive(Debug, Deserialize, Clone)]
pub struct UpdateProperties {
    pub name: Option<String>,
    pub description: Option<String>,
    pub location: Option<LocationTriplet>,
    pub website: Option<WebAddress>,
    pub icon: Option<EntryHash>,
    pub email: Option<String>,
    pub editors: Option<Vec<AgentPubKey>>,
    pub published_at: Option<u64>,
    pub last_updated: Option<u64>,
    pub metadata: Option<BTreeMap<String, RmpvValue>>,
}
pub type UpdateInput = UpdateEntityInput<UpdateProperties>;

#[hdk_extern]
pub fn update_publisher(input: UpdateInput) -> ExternResult<Entity<PublisherEntry>> {
    debug!("Updating Publisher: {}", input.base );
    let props = input.properties.clone();

    let entity = update_entity(
	&input.base,
	|mut current : PublisherEntry, _| {
	    current.name = props.name
		.unwrap_or( current.name );
	    current.description = props.description
		.or( current.description );
	    current.location = props.location
		.unwrap_or( current.location );
	    current.website = props.website
		.unwrap_or( current.website );
	    current.icon = props.icon
		.unwrap_or( current.icon );
	    current.email = props.email
		.or( current.email );
	    current.author = agent_id()?;
	    current.published_at = props.published_at
		.unwrap_or( current.published_at );
	    current.last_updated = props.last_updated
		.unwrap_or( current.last_updated );
	    current.metadata = props.metadata
		.unwrap_or( current.metadata );

	    Ok( current )
	})?;

    Ok( entity )
}


#[derive(Debug, Deserialize)]
pub struct DeprecateInput {
    pub base: ActionHash,
    pub message: String,
}

#[hdk_extern]
pub fn deprecate_publisher(input: DeprecateInput) -> ExternResult<Entity<PublisherEntry>> {
    debug!("Deprecating publisher: {}", input.base );
    let entity = update_entity(
	&input.base,
	|mut current : PublisherEntry, _| {
	    current.deprecation = Some(DeprecationNotice {
		message: input.message.to_owned(),
		recommended_alternatives: None,
	    });

	    Ok( current )
	})?;

    Ok( entity )
}


#[derive(Debug, Deserialize)]
pub struct UndeprecateInput {
    pub base: ActionHash,
}

#[hdk_extern]
pub fn undeprecate_publisher(input: UndeprecateInput) -> ExternResult<Entity<PublisherEntry>> {
    debug!("Undeprecating publisher: {}", input.base );
    let entity = update_entity(
	&input.base,
	|mut current : PublisherEntry, _| {
	    current.deprecation = None;

	    Ok( current )
	})?;

    Ok( entity )
}
