use crate::{
    hdk,
};

use std::collections::BTreeMap;
use hdk::prelude::*;
use hdk_extensions::{
    agent_id,
};
use appstore::{
    EntryTypes,
    LinkTypes,
    RmpvValue,

    HRL,
    BundleHashes,
    AppVersionEntry,

    hc_crud::{
        now, create_entity, get_entity, update_entity, delete_entity,
        Entity,
        EntityId,
        GetEntityInput, UpdateEntityInput,
    },
};



#[derive(Debug, Deserialize)]
pub struct CreateInput {
    pub version: String,
    pub for_app: EntityId,
    pub apphub_hrl: HRL,
    pub apphub_hrl_hash: EntryHash,
    pub bundle_hashes: BundleHashes,

    // optional
    pub published_at: Option<u64>,
    pub last_updated: Option<u64>,
    pub metadata: Option<BTreeMap<String, RmpvValue>>,
}

#[hdk_extern]
pub fn create_app_version(input: CreateInput) -> ExternResult<Entity<AppVersionEntry>> {
    debug!("Creating AppVersion: {}", input.version );
    let pubkey = agent_id()?;
    let default_now = now()?;

    let app_version = AppVersionEntry {
	version: input.version,
	for_app: input.for_app.clone(),
	apphub_hrl: input.apphub_hrl,
	apphub_hrl_hash: input.apphub_hrl_hash,
	bundle_hashes: input.bundle_hashes,

	author: pubkey,
	published_at: input.published_at
	    .unwrap_or( default_now ),
	last_updated: input.last_updated
	    .unwrap_or( default_now ),

	metadata: input.metadata
	    .unwrap_or( BTreeMap::new() ),
    };
    let entity = create_entity( &app_version )?;

    { // Link from App
	entity.link_from( &input.for_app, LinkTypes::AppToAppVersion, None )?;
    }

    Ok( entity )
}


#[hdk_extern]
pub fn get_app_version(input: GetEntityInput) -> ExternResult<Entity<AppVersionEntry>> {
    debug!("Get app_version: {}", input.id );
    let entity : Entity<AppVersionEntry> = get_entity( &input.id )?;

    Ok(	entity )
}


#[derive(Debug, Deserialize, Clone)]
pub struct UpdateProperties {
    pub version: Option<String>,
    pub published_at: Option<u64>,
    pub last_updated: Option<u64>,
    pub metadata: Option<BTreeMap<String, RmpvValue>>,
}
pub type UpdateInput = UpdateEntityInput<UpdateProperties>;

#[hdk_extern]
pub fn update_app_version(input: UpdateInput) -> ExternResult<Entity<AppVersionEntry>> {
    debug!("Updating AppVersion: {}", input.base );
    let props = input.properties.clone();

    let entity = update_entity(
	&input.base,
	|mut current : AppVersionEntry, _| {
	    current.version = props.version
		.unwrap_or( current.version );
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
pub struct DeleteInput {
    pub base: ActionHash,
}

#[hdk_extern]
pub fn delete_app_version(input: DeleteInput) -> ExternResult<ActionHash> {
    debug!("Deleting AppVersion: {}", input.base );
    let delete_hash = delete_entity::<AppVersionEntry, EntryTypes>( &input.base )?;

    Ok( delete_hash )
}
