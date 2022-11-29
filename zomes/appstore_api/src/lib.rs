mod constants;
mod publisher;

use hdk::prelude::*;
pub use appstore::{
    LinkTypes,
    EntryTypes,

    AppEntry,
    PublisherEntry,

    GetEntityInput, EntityId,
    AppResult, Response, EntityResponse,
    composition, catch,

    AppError,
};
pub use constants::{
    ENTITY_MD,
    VALUE_MD,

    ANCHOR_AGENTS,
    ANCHOR_PUBLISHERS,
};



#[hdk_extern]
fn init(_: ()) -> ExternResult<InitCallbackResult> {
    Ok(InitCallbackResult::Pass)
}


#[hdk_extern]
fn whoami(_: ()) -> ExternResult<Response<AgentInfo>> {
    Ok(composition( agent_info()?, VALUE_MD ))
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

// #[hdk_extern]
// fn get_publishers_by_filter( input: FilterInput ) -> ExternResult<Response<Vec<Entity<PublisherEntry>>>> {
//     let collection = catch!( types::get_by_filter( LinkTypes::App, input.filter, input.keyword ) );

//     Ok(composition(
// 	collection.into_iter()
// 	    .filter(|entity: &Entity<PublisherEntry>| {
// 		entity.content.deprecation.is_none()
// 	    })
// 	    .collect(),
// 	ENTITY_COLLECTION_MD
//     ))
// }

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
