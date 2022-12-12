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
    Response,

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
	let (_, pathhash ) = hc_utils::path( ANCHOR_AGENTS, vec![
	    hc_utils::agentid()?,
	    ANCHOR_APPS.to_string(),
	]);
	entity.link_from( &pathhash, LinkTypes::App, None )?;
    }
    { // Path via Publisher's Apps
	let (_, pathhash) = hc_utils::path( ANCHOR_PUBLISHERS, vec![
	    input.publisher.to_string(),
	    ANCHOR_APPS.to_string(),
	]);
	entity.link_from( &pathhash, LinkTypes::App, None )?;
    }
    { // Path via All Apps
	let (_, pathhash) = hc_utils::path( ANCHOR_APPS, vec![
	    entity.id.clone(),
	]);
	entity.link_from( &pathhash, LinkTypes::App, None )?;
    }

    Ok( entity )
}


pub fn get(input: GetEntityInput) -> AppResult<Entity<AppEntry>> {
    debug!("Get app: {}", input.id );
    let entity : Entity<AppEntry> = get_entity( &input.id )?;

    Ok(	entity )
}


#[derive(Debug, Serialize, Deserialize)]
pub struct GetWebHappPackageInput {
    pub name: String,
    pub happ_release_id: EntryHash,
    pub gui_release_id: EntryHash,
}

impl Into<rmpv::Value> for GetWebHappPackageInput {
    fn into(self) -> rmpv::Value {
	let serialized = rmp_serde::to_vec( &self ).unwrap();
	rmp_serde::from_slice( &serialized ).unwrap()
    }
}

pub fn get_package(input: GetEntityInput) -> AppResult<Vec<u8>> {
    let entity = get( input )?;

    let response = call(
	CallTargetCell::OtherRole("portal".into()),
	"portal_api",
	"remote_call".into(),
	None, // CapSecret
	portal_types::RemoteCallInput {
	    dna: entity.content.devhub_address.dna,
	    zome: "happ_library".to_string(),
	    function: "get_webhapp_package".to_string(),
	    payload: GetWebHappPackageInput {
		name: entity.content.name,
		happ_release_id: entity.content.devhub_address.happ,
		gui_release_id: entity.content.devhub_address.gui,
	    }.into(),
	}
    )?;
    let result = hc_utils::zome_call_response_as_result( response )?;
    let essence_resp : Response<Vec<u8>> = result.decode()?;

    Ok( essence_resp.as_result()? )
}


#[derive(Debug, Deserialize, Clone)]
pub struct UpdateProperties {
    pub name: Option<String>,
    pub description: Option<String>,
    pub icon: Option<EntityId>,
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

	    Ok( current )
	})?;

    // let previous = previous.unwrap();

    Ok( entity )
}
