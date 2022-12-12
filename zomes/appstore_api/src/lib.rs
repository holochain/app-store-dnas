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
};
pub use constants::{
    ENTITY_MD,
    ENTITY_COLLECTION_MD,
    VALUE_MD,

    ANCHOR_AGENTS,
    ANCHOR_PUBLISHERS,
    ANCHOR_APPS,
};



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
fn get_app_package(input: GetEntityInput) -> ExternResult<Response<Vec<u8>>> {
    let bytes = catch!( app::get_package( input ) );

    Ok(composition( bytes, VALUE_MD ))
}

#[hdk_extern]
fn update_app(input: app::UpdateInput) -> ExternResult<EntityResponse<AppEntry>> {
    let entity = catch!( app::update( input ) );

    Ok(composition( entity, ENTITY_MD ))
}

// #[hdk_extern]
// fn deprecate_publisher(input: publisher::AppDeprecateInput) -> ExternResult<EntityResponse<PublisherEntry>> {
//     let entity = catch!( publisher::deprecate_publisher( input ) );

//     Ok(composition( entity, ENTITY_MD ))
// }

// #[hdk_extern]
// fn get_publishers(input: GetAgentItemsInput) -> ExternResult<Response<Vec<Entity<PublisherEntry>>>> {
//     let (base_path, _) = types::create_path( &agent_path_base( input.agent ), vec![ ANCHOR_PUBLISHERS ] );
//     let collection = catch!( types::get_entities_for_path_filtered( base_path, LinkTypes::App, None, |items : Vec<Entity<PublisherEntry>>| {
// 	Ok( items.into_iter()
// 	    .filter(|entity| {
// 		entity.content.deprecation.is_none()
// 	    })
// 	    .collect() )
//     }) );

//     Ok(composition( collection, ENTITY_COLLECTION_MD ))
// }

// #[hdk_extern]
// fn get_my_publishers(_:()) -> ExternResult<Response<Vec<Entity<PublisherEntry>>>> {
//     get_publishers( GetAgentItemsInput {
// 	agent: None
//     })
// }


#[derive(Debug, Serialize, Deserialize)]
pub struct GetForAgentInput {
    pub for_agent: AgentPubKey,
}

#[hdk_extern]
fn get_publishers_for_agent( input: GetForAgentInput ) -> ExternResult<Response<Vec<Entity<PublisherEntry>>>> {
    let (_, pathhash ) = hc_utils::path( ANCHOR_AGENTS, vec![
	input.for_agent.to_string(), ANCHOR_PUBLISHERS.to_string(),
    ]);
    let collection = catch!( match hc_crud::get_entities( &pathhash, LinkTypes::Publisher, None ) {
	Ok(c) => Ok(c),
	Err(e) => Err(e)?,
    });

    Ok(composition(
	collection,
	    // .into_iter()
	    // .filter(|entity: &Entity<PublisherEntry>| {
	    // 	entity.content.deprecation.is_none()
	    // })
	    // .collect(),
	ENTITY_COLLECTION_MD
    ))
}

// #[hdk_extern]
// fn get_publishers_by_tags( input: Vec<String> ) -> ExternResult<Response<Vec<Entity<PublisherEntry>>>> {
//     let list = catch!( types::get_by_tags( LinkTypes::App, input ) );

//     Ok(composition( list.into_iter()
// 		    .filter(|entity: &Entity<PublisherEntry>| {
// 			entity.content.deprecation.is_none()
// 		    })
// 		    .collect(), VALUE_MD ))
// }

// #[hdk_extern]
// fn get_all_publishers(_:()) -> ExternResult<Response<Vec<Entity<PublisherEntry>>>> {
//     let (base_path, _) = types::create_path( ANCHOR_PUBLISHERS, Vec::<String>::new() );
//     let collection = catch!( types::get_entities_for_path_filtered( base_path, LinkTypes::App, None, |items : Vec<Entity<PublisherEntry>>| {
// 	Ok( items.into_iter()
// 	    .filter(|entity| {
// 		entity.content.deprecation.is_none()
// 	    })
// 	    .collect() )
//     }) );

//     Ok(composition( collection, ENTITY_COLLECTION_MD ))
// }
