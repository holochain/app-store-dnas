use std::collections::BTreeMap;
use hdk::prelude::*;
use hc_crud::{
    now, create_entity, get_entity, update_entity,
    Entity,
};
use appstore::{
    LinkTypes,
    RmpvValue,

    HRL,
    DeprecationNotice,
    AppEntry,

    EntityId,
    GetEntityInput, UpdateEntityInput,
};
use crate::{
    ANCHOR_AGENTS,
    ANCHOR_PUBLISHERS,
    ANCHOR_APPS,
};



#[derive(Debug, Deserialize)]
pub struct CreateInput {
    pub title: String,
    pub subtitle: String,
    pub description: String,
    pub icon: EntryHash,
    pub publisher: EntityId,
    pub apphub_hrl: HRL,

    // optional
    pub editors: Option<Vec<AgentPubKey>>,

    pub published_at: Option<u64>,
    pub last_updated: Option<u64>,
    pub metadata: Option<BTreeMap<String, RmpvValue>>,
}

#[hdk_extern]
pub fn create_app(mut input: CreateInput) -> ExternResult<Entity<AppEntry>> {
    debug!("Creating App: {}", input.title );
    let pubkey = agent_info()?.agent_initial_pubkey;
    let default_now = now()?;
    let default_editors = vec![ pubkey.clone() ];

    if let Some(ref mut editors) = input.editors {
	if !editors.contains( &pubkey ) {
	    editors.splice( 0..0, default_editors.clone() );
	}
    }

    let app = AppEntry {
	title: input.title,
	subtitle: input.subtitle,
	description: input.description,
	icon: input.icon,
	publisher: input.publisher.clone(),
	apphub_hrl: input.apphub_hrl,

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


#[hdk_extern]
pub fn get_app(input: GetEntityInput) -> ExternResult<Entity<AppEntry>> {
    debug!("Get app: {}", input.id );
    let entity : Entity<AppEntry> = get_entity( &input.id )?;

    Ok(	entity )
}


#[derive(Debug, Deserialize, Clone)]
pub struct UpdateProperties {
    pub title: Option<String>,
    pub subtitle: Option<String>,
    pub description: Option<String>,
    pub icon: Option<EntryHash>,
    pub apphub_hrl: Option<HRL>,
    pub editors: Option<Vec<AgentPubKey>>,
    pub published_at: Option<u64>,
    pub last_updated: Option<u64>,
    pub metadata: Option<BTreeMap<String, RmpvValue>>,
}
pub type UpdateInput = UpdateEntityInput<UpdateProperties>;

#[hdk_extern]
pub fn update_app(input: UpdateInput) -> ExternResult<Entity<AppEntry>> {
    debug!("Updating App: {}", input.base );
    let props = input.properties.clone();
    let mut previous : Option<AppEntry> = None;

    let entity = update_entity(
	&input.base,
	|mut current : AppEntry, _| {
	    previous = Some(current.clone());

	    current.title = props.title
		.unwrap_or( current.title );
	    current.subtitle = props.subtitle
		.unwrap_or( current.subtitle );
	    current.description = props.description
		.unwrap_or( current.description );
	    current.apphub_hrl = props.apphub_hrl
		.unwrap_or( current.apphub_hrl );
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


#[derive(Debug, Deserialize)]
pub struct DeprecateInput {
    pub base: ActionHash,
    pub message: String,
}

#[hdk_extern]
pub fn deprecate_app(input: DeprecateInput) -> ExternResult<Entity<AppEntry>> {
    debug!("Deprecating hApp: {}", input.base );
    let entity = update_entity(
	&input.base,
	|mut current : AppEntry, _| {
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
pub fn undeprecate_app(input: UndeprecateInput) -> ExternResult<Entity<AppEntry>> {
    debug!("Deprecating hApp: {}", input.base );
    let entity = update_entity(
	&input.base,
	|mut current : AppEntry, _| {
	    current.deprecation = None;

	    Ok( current )
	})?;

    Ok( entity )
}
