mod constants;
mod publisher;
mod app;

use std::collections::BTreeMap;
use hdk::prelude::*;
use hc_crud::{
    EntryModel,
};
pub use appstore::{
    LinkTypes,
    EntryTypes,

    AppEntry,
    PublisherEntry,
    ModeratorActionEntry,
    GroupAnchorEntry,

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
use coop_content_sdk::{
    GroupEntry,
    hdi_extensions,
    hdk_extensions,
    call_local_zome_decode,
    create_group, get_group, update_group,
    register_content_to_group,
    register_content_update_to_group,
    get_all_group_content_latest,
};
use hdi_extensions::{
    guest_error,
};
use hdk_extensions::{
    UpdateEntryInput,
    exists,
    must_get,
    follow_evolutions,
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


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetModeratorActionsInput {
    pub group_id: ActionHash,
    pub app_id: ActionHash,
}

#[hdk_extern]
fn get_moderator_actions(input: GetModeratorActionsInput) -> ExternResult<Response<Vec<Vec<ModeratorActionEntry>>>> {
    // - Find group anchor
    // - Find moderator action link with tag 'app::<app_id>'
    // - Follow evolutions for group
    let group_anchor_entry = GroupAnchorEntry {
        group_id: input.group_id.clone(),
    };
    let group_anchor_hash = hash_entry( &group_anchor_entry )?;

    let tag = format!("app::{}", input.app_id );
    let moderator_action_links = get_links(
        group_anchor_hash,
        LinkTypes::ModeratorAction,
        Some( LinkTag::new( tag ) ),
    )?;

    let action_histories = moderator_action_links
        .into_iter()
        .map(|link| {
            type Response = Vec<ActionHash>;
            call_local_zome_decode!(
                Response,
                "coop_content_csr",
                "get_group_content_evolutions",
                coop_content_sdk::GetGroupContentInput {
                    group_id: input.group_id.clone(),
                    content_id: link.target,
                    full_trace: None,
                }
            )
        })
        .filter_map(|result| {
            result.ok()
        })
        .map(|history| {
            history
                .into_iter()
                .filter_map(|addr| {
                    ModeratorActionEntry::try_from( must_get( &addr ).ok()? ).ok()
                })
                .collect()
        })
        .collect();

    Ok(composition(
	action_histories,
	VALUE_MD
    ))
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RemoveAppInput {
    pub group_id: ActionHash,
    pub app_id: ActionHash,
    pub message: String,
    pub metadata: Option<BTreeMap<String, serde_yaml::Value>>,
}

#[hdk_extern]
fn remove_app(input: RemoveAppInput) -> ExternResult<Response<Entity<ModeratorActionEntry>>> {
    // - Create a moderator action record
    // - If not exists, create group anchor
    // - Link moderator action to group anchor with tag 'app::<app_id>'
    // - Add moderator action to group
    let mut required_metadata = BTreeMap::new();
    required_metadata.insert("remove".into(), (true).into() );

    let group_rev = follow_evolutions( &input.group_id )?.last().unwrap().to_owned();
    let moderator_action_entry = ModeratorActionEntry {
        group_id: (input.group_id.clone(), group_rev),
        message: input.message,
        subject_id: input.app_id.clone(),
        metadata: match input.metadata {
            Some(mut metadata) => {
                metadata.extend( required_metadata );
                metadata
            },
            None => required_metadata,
        },
    };
    let entity = hc_crud::create_entity( &moderator_action_entry )?;

    let group_anchor_entry = GroupAnchorEntry {
        group_id: input.group_id,
    };
    let group_anchor_hash = hash_entry( &group_anchor_entry )?;

    if !exists( &group_anchor_hash )? {
        let group_anchor_addr = create_entry( group_anchor_entry.to_input() )?;

        register_content_to_group!({
            entry: group_anchor_entry,
            target: group_anchor_addr,
        })?;
    }

    register_content_to_group!({
        entry: moderator_action_entry,
        target: entity.id.clone(),
    })?;

    let tag = format!("app::{}", input.app_id );
    create_link( group_anchor_hash, entity.id.clone(), LinkTypes::ModeratorAction, tag.into_bytes() )?;

    Ok(composition(
	entity,
	ENTITY_MD
    ))
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnremoveAppInput {
    pub moderator_action_base: ActionHash,
    pub message: String,
    pub metadata: Option<BTreeMap<String, serde_yaml::Value>>,
}

#[hdk_extern]
fn unremove_app(input: UnremoveAppInput) -> ExternResult<Response<ActionHash>> {
    // - Update moderator action record
    // - Add moderator action update to group
    let mut ma_entry = ModeratorActionEntry::try_from( must_get( &input.moderator_action_base )? )?;
    let group_rev = follow_evolutions( &ma_entry.group_id.0.clone() )?.last().unwrap().to_owned();
    ma_entry.group_id = (ma_entry.group_id.0, group_rev);
    ma_entry.message = input.message;
    ma_entry.metadata.insert("remove".to_string(), serde_yaml::Value::Bool(false) );

    let action_hash = update_entry( input.moderator_action_base, ma_entry.to_input() )?;

    register_content_update_to_group!({
        entry: ma_entry,
        target: action_hash.clone(),
    })?;

    Ok(composition(
	action_hash,
	VALUE_MD
    ))
}



//
// Group CRUD
//
#[hdk_extern]
pub fn create_group(group: GroupEntry) -> ExternResult<ActionHash> {
    debug!("Creating new group entry: {:#?}", group );
    let action_hash = create_group!( group )?;

    Ok( action_hash )
}


#[hdk_extern]
pub fn get_group(id: ActionHash) -> ExternResult<GroupEntry> {
    debug!("Creating new group entry: {:#?}", id );
    let group = get_group!( id )?;

    Ok( group )
}


#[hdk_extern]
pub fn update_group(input: UpdateEntryInput<GroupEntry>) -> ExternResult<ActionHash> {
    debug!("Update group: {:#?}", input );
    let action_hash = update_group!({
        base: input.base,
        entry: input.entry,
    })?;

    Ok( action_hash )
}


#[hdk_extern]
fn viewpoint_get_all_apps(group_id: ActionHash) -> ExternResult<Response<Vec<Entity<AppEntry>>>> {
    // - Derive group anchor
    // - Get moderator action links
    // - Get all group content
    let removed_app_ids : Vec<ActionHash> = get_all_group_content_latest!({
        group_id: group_id.clone(),
    })?.into_iter()
        .filter_map(|(_origin, latest)| {
            debug!("Get latest group entry: {}", group_id );
            let addr = latest.into_action_hash()?;
            let record = must_get( &addr ).ok()?;
            Some( ModeratorActionEntry::try_from( record ).ok()? )
        })
        .filter_map(|moderator_action| {
            match moderator_action.metadata.get("remove")? {
                serde_yaml::Value::Bool(value) => match value == &true {
                    true => Some( moderator_action.subject_id ),
                    false => None,
                },
                _ => None,
            }
        })
        .collect();
    let apps = get_all_apps(())?.as_result()
        .map_err(|err| guest_error!(format!("{:?}", err )))?
        .into_iter()
        .filter(|entity| !removed_app_ids.contains( &entity.id ) )
        .collect();

    Ok(composition(
	apps,
	ENTITY_COLLECTION_MD
    ))
}
