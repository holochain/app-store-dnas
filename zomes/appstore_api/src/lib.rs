use std::collections::BTreeMap;
mod constants;
mod publisher;
mod app;

use hdk::prelude::*;
pub use appstore::{
    LinkTypes,
    EntryTypes,

    AppEntry,
    PublisherEntry,

    GetEntityInput, EntityId,
    AppResult, Response, EntityResponse, Entity,
    composition, catch,

    AppError,
    UserError,
};
pub use portal_types::{
    HostEntry,
    DnaZomeFunction,
};
pub use constants::{
    ENTITY_MD,
    ENTITY_COLLECTION_MD,
    VALUE_MD,

    ANCHOR_AGENTS,
    ANCHOR_PUBLISHERS,
    ANCHOR_APPS,
};

#[derive(Debug, Serialize, Deserialize)]
pub struct GetForAgentInput {
    pub for_agent: AgentPubKey,
}



#[derive(Debug, Deserialize, Serialize, SerializedBytes)]
pub struct DnaProperties {
    pub dna_hash_alias: BTreeMap<String,holo_hash::DnaHash>, // DnaHashAlias,
}

pub fn dna_properties () -> AppResult<DnaProperties> {
    Ok( dna_info()?.properties.try_into()? )
}



#[hdk_extern]
fn init(_: ()) -> ExternResult<InitCallbackResult> {
    Ok(InitCallbackResult::Pass)
}


#[hdk_extern]
fn whoami(_: ()) -> ExternResult<Response<AgentInfo>> {
    Ok(composition( agent_info()?, VALUE_MD ))
}

pub fn save_bytes(bytes: &Vec<u8>) -> AppResult<EntryHash> {
    let response = call(
	CallTargetCell::Local,
	"mere_memory_api",
	"save_bytes".into(),
	None, // CapSecret
	bytes
    )?;

    let result = hc_utils::zome_call_response_as_result( response )?;
    let essence_resp : Response<EntryHash> = result.decode()?;
    debug!("Decoded result: {:#?}", essence_resp );

    Ok( essence_resp.as_result()? )
}

// Publisher
#[hdk_extern]
fn create_publisher(input: publisher::CreateInput) -> ExternResult<EntityResponse<PublisherEntry>> {
    let entity = catch!( publisher::create( input ) );

    Ok(composition( entity, ENTITY_MD ))
}

#[hdk_extern]
fn get_publisher(input: GetEntityInput) -> ExternResult<EntityResponse<PublisherEntry>> {
    let entity = catch!( publisher::get( input ) );

    Ok(composition( entity, ENTITY_MD ))
}

#[hdk_extern]
fn update_publisher(input: publisher::UpdateInput) -> ExternResult<EntityResponse<PublisherEntry>> {
    let entity = catch!( publisher::update( input ) );

    Ok(composition( entity, ENTITY_MD ))
}

#[hdk_extern]
fn get_publishers_for_agent(input: GetForAgentInput) -> ExternResult<Response<Vec<Entity<PublisherEntry>>>> {
    let (_, pathhash ) = hc_utils::path( ANCHOR_AGENTS, vec![
	input.for_agent.to_string(), ANCHOR_PUBLISHERS.to_string(),
    ]);
    let collection = catch!(
	hc_crud::get_entities( &pathhash, LinkTypes::Publisher, None )
	    .map_err(|e| e.into())
    );

    Ok(composition(
	collection,
	ENTITY_COLLECTION_MD
    ))
}

#[hdk_extern]
fn get_my_publishers(_:()) -> ExternResult<Response<Vec<Entity<PublisherEntry>>>> {
    get_publishers_for_agent( GetForAgentInput {
	for_agent: hc_utils::agentpubkey()?,
    })
}

#[hdk_extern]
fn get_all_publishers(_: ()) -> ExternResult<Response<Vec<Entity<PublisherEntry>>>> {
    let (_, pathhash ) = hc_utils::path_base( ANCHOR_PUBLISHERS );
    let collection = catch!(
	hc_crud::get_entities( &pathhash, LinkTypes::Publisher, None )
	    .map_err(|e| e.into())
    );

    Ok(composition(
	collection,
	ENTITY_COLLECTION_MD
    ))
}


// App
#[hdk_extern]
fn create_app(input: app::CreateInput) -> ExternResult<EntityResponse<AppEntry>> {
    let entity = catch!( app::create( input ) );

    Ok(composition( entity, ENTITY_MD ))
}

#[hdk_extern]
fn get_app(input: GetEntityInput) -> ExternResult<EntityResponse<AppEntry>> {
    let entity = catch!( app::get( input ) );

    Ok(composition( entity, ENTITY_MD ))
}

#[hdk_extern]
fn update_app(input: app::UpdateInput) -> ExternResult<EntityResponse<AppEntry>> {
    let entity = catch!( app::update( input ) );

    Ok(composition( entity, ENTITY_MD ))
}

#[hdk_extern]
fn get_apps_for_agent(input: GetForAgentInput) -> ExternResult<Response<Vec<Entity<AppEntry>>>> {
    let (_, pathhash ) = hc_utils::path( ANCHOR_AGENTS, vec![
	input.for_agent.to_string(), ANCHOR_APPS.to_string(),
    ]);
    let collection = catch!(
	hc_crud::get_entities( &pathhash, LinkTypes::App, None )
	    .map_err(|e| e.into())
    );

    Ok(composition(
	collection,
	ENTITY_COLLECTION_MD
    ))
}

#[hdk_extern]
fn get_my_apps(_:()) -> ExternResult<Response<Vec<Entity<AppEntry>>>> {
    get_apps_for_agent( GetForAgentInput {
	for_agent: hc_utils::agentpubkey()?,
    })
}

#[hdk_extern]
fn get_all_apps(_: ()) -> ExternResult<Response<Vec<Entity<AppEntry>>>> {
    let (_, pathhash ) = hc_utils::path_base( ANCHOR_APPS );
    let collection = catch!(
	hc_crud::get_entities( &pathhash, LinkTypes::App, None )
	    .map_err(|e| e.into())
    );

    Ok(composition(
	collection,
	ENTITY_COLLECTION_MD
    ))
}


#[derive(Debug, Serialize)]
pub struct GetInput {
    pub dna: holo_hash::DnaHash,
}


fn handler_get_registered_hosts(alias: String) -> AppResult<Response<Vec<Entity<HostEntry>>>> {
    let props = dna_properties()?;

    let response = call(
	CallTargetCell::OtherRole(String::from("portal")),
	"portal_api",
	"get_registered_hosts".into(),
	None, // CapSecret
	GetInput {
	    dna: props.dna_hash_alias.get(&alias)
		.ok_or( UserError::CustomError(format!("Unknown alias '{}'", alias )) )?
		.to_owned(),
	}
    )?;

    let result = hc_utils::zome_call_response_as_result( response )?;
    Ok( result.decode()? )
}

#[hdk_extern]
fn get_registered_hosts(alias: String) -> ExternResult<Response<Vec<Entity<HostEntry>>>> {
    let hosts = catch!( handler_get_registered_hosts(alias) );

    Ok( hosts )
}



fn handler_get_hosts_for_zome_function(dna_alias: String, zome: ZomeName, function: FunctionName) -> AppResult<Response<Vec<Entity<HostEntry>>>> {
    let props = dna_properties()?;

    let response = call(
	CallTargetCell::OtherRole(String::from("portal")),
	"portal_api",
	"get_hosts_for_zome_function".into(),
	None, // CapSecret
	DnaZomeFunction {
	    dna: props.dna_hash_alias.get(&dna_alias)
		.ok_or( UserError::CustomError(format!("Unknown alias '{}'", dna_alias )) )?
		.to_owned(),
	    zome,
	    function,
	}
    )?;

    let result = hc_utils::zome_call_response_as_result( response )?;
    Ok( result.decode()? )
}

#[derive(Debug, Deserialize)]
pub struct GetHostsForInput {
    pub dna: String,
    pub zome: ZomeName,
    pub function: FunctionName,
}

#[hdk_extern]
fn get_hosts_for_zome_function(input: GetHostsForInput) -> ExternResult<Response<Vec<Entity<HostEntry>>>> {
    let hosts = catch!( handler_get_hosts_for_zome_function(input.dna, input.zome, input.function) );

    Ok( hosts )
}


fn handler_get_dna_hash(alias: String) -> AppResult<holo_hash::DnaHash> {
    let props = dna_properties()?;

    Ok(
	props.dna_hash_alias.get(&alias)
	    .ok_or( UserError::CustomError(format!("Unknown alias '{}'", alias )) )?
	    .to_owned()
    )
}

#[hdk_extern]
fn get_dna_hash(alias: String) -> ExternResult<Response<holo_hash::DnaHash>> {
    let hash = catch!( handler_get_dna_hash(alias) );

    Ok(composition( hash, VALUE_MD ))
}
