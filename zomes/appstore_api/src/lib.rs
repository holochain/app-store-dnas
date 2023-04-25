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
fn deprecate_publisher(input: publisher::DeprecateInput) -> ExternResult<EntityResponse<PublisherEntry>> {
    let entity = catch!( publisher::deprecate( input ) );

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
    let collection = collection.into_iter()
	.filter(|entity : &Entity<PublisherEntry>| {
	    entity.content.deprecation.is_none()
	})
	.collect();

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
    let collection = collection.into_iter()
	.filter(|entity : &Entity<PublisherEntry>| {
	    entity.content.deprecation.is_none()
	})
	.collect();

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
fn deprecate_app(input: app::DeprecateInput) -> ExternResult<EntityResponse<AppEntry>> {
    let entity = catch!( app::deprecate( input ) );

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
    let collection = collection.into_iter()
	.filter(|entity : &Entity<AppEntry>| {
	    entity.content.deprecation.is_none()
	})
	.collect();

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
    let collection = collection.into_iter()
	.filter(|entity : &Entity<AppEntry>| {
	    entity.content.deprecation.is_none()
	})
	.collect();

    Ok(composition(
	collection,
	ENTITY_COLLECTION_MD
    ))
}
