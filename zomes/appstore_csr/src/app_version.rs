use crate::{
    hdk,
    coop_content_sdk,
    GetForAppInput,
};

use std::collections::BTreeMap;
use hdk::prelude::*;
use hdk_extensions::{
    must_get,
};
use appstore::{
    EntryTypes,
    RmpvValue,

    HRL,
    BundleHashes,
    AppVersionEntry,

    hc_crud::{
        now, create_entity, update_entity, delete_entity,
        Entity, EntityId, EntryModel,
        GetEntityInput, UpdateEntityInput,
    },
};
use coop_content_sdk::{
    register_content_to_group,
    register_content_update_to_group,
    get_group_content_latest,
    get_all_group_content_latest,
    GroupRef,
};



#[derive(Debug, Deserialize)]
pub struct CreateInput {
    pub version: String,
    pub for_app: EntityId,
    pub apphub_hrl: HRL,
    pub apphub_hrl_hash: EntryHash,
    pub bundle_hashes: BundleHashes,

    // optional
    pub editors_group_id: Option<(ActionHash, ActionHash)>,
    pub published_at: Option<u64>,
    pub last_updated: Option<u64>,
    pub metadata: Option<BTreeMap<String, RmpvValue>>,
}

#[hdk_extern]
pub fn create_app_version(input: CreateInput) -> ExternResult<Entity<AppVersionEntry>> {
    debug!("Creating AppVersion: {}", input.version );
    let default_now = now()?;

    // Get the latest group ref based on the parent app
    let editors_group_id = match input.editors_group_id {
        None => {
            let app_entry = crate::app::get_app(GetEntityInput {
                id: input.for_app.clone(),
            })?.content;
            let editors_group_id = app_entry.group_ref().0;
            (
                editors_group_id.clone(),
                crate::group::get_group( editors_group_id )?.action,
            )
        },
        Some(editors_group_id) => editors_group_id,
    };

    let app_version = AppVersionEntry {
	version: input.version,
	for_app: input.for_app.clone(),
	apphub_hrl: input.apphub_hrl,
	apphub_hrl_hash: input.apphub_hrl_hash,
	bundle_hashes: input.bundle_hashes,
	editors_group_id: editors_group_id,

	published_at: input.published_at
	    .unwrap_or( default_now ),
	last_updated: input.last_updated
	    .unwrap_or( default_now ),

	metadata: input.metadata
	    .unwrap_or( BTreeMap::new() ),
    };
    let entity = create_entity( &app_version )?;

    // Link from group
    register_content_to_group!({
        entry: app_version,
        target: entity.id.clone(),
    })?;

    Ok( entity )
}


#[hdk_extern]
pub fn get_app_version(input: GetEntityInput) -> ExternResult<Entity<AppVersionEntry>> {
    debug!("Get app_version: {}", input.id );
    let app_version_origin : AppVersionEntry = must_get( &input.id )?.try_into()?;

    let latest_addr = get_group_content_latest!({
        group_id: app_version_origin.group_ref().0,
        content_id: input.id.clone().into(),
    })?;

    let app_version : AppVersionEntry = must_get( &latest_addr )?.try_into()?;

    Ok(
	Entity {
            id: input.id,
            address: hash_entry( app_version.clone() )?,
            action: latest_addr,
            ctype: app_version.get_type(),
            content: app_version,
        }
    )
}


#[derive(Debug, Deserialize, Clone)]
pub struct UpdateProperties {
    pub version: Option<String>,
    pub for_app: Option<EntityId>,
    pub apphub_hrl: Option<HRL>,
    pub apphub_hrl_hash: Option<EntryHash>,
    pub bundle_hashes: Option<BundleHashes>,
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
	    current.for_app = props.for_app
		.unwrap_or( current.for_app );
	    current.apphub_hrl = props.apphub_hrl
		.unwrap_or( current.apphub_hrl );
	    current.apphub_hrl_hash = props.apphub_hrl_hash
		.unwrap_or( current.apphub_hrl_hash );
	    current.bundle_hashes = props.bundle_hashes
		.unwrap_or( current.bundle_hashes );

            // Automatically update the group ref to latest
            update_app_version_editors_group_ref( &mut current )?;

	    current.published_at = props.published_at
		.unwrap_or( current.published_at );
	    current.last_updated = props.last_updated
		.unwrap_or( current.last_updated );
	    current.metadata = props.metadata
		.unwrap_or( current.metadata );

	    Ok( current )
	})?;

    register_content_update_to_group!({
        entry: entity.content.clone(),
        target: entity.action.clone(),
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


/// Update the group ref to latest
fn update_app_version_editors_group_ref(app_version: &mut AppVersionEntry) -> ExternResult<()> {
    let group_entity = crate::group::get_group( app_version.group_ref().0 )?;

    app_version.editors_group_id = (
        app_version.group_ref().0, group_entity.action
    );

    Ok(())
}


/// Get App Versions that belong to the given App ID
#[hdk_extern]
pub fn get_app_versions_for_app(input: GetForAppInput) -> ExternResult<Vec<Entity<AppVersionEntry>>> {
    let app_entry = crate::app::get_app(GetEntityInput {
        id: input.for_app.clone(),
    })?.content;

    // TODO: what happens if multiple publishers used this group?  Do we care?  Will it just be by
    // convention that you shouldn't reuse groups
    Ok(
        get_all_group_content_latest!({
            group_id: app_entry.group_ref().0,
        })?.into_iter()
            .filter_map(|(origin, latest)| {
                let addr = latest.into_action_hash()?;
                let record = must_get( &addr ).ok()?;
                let app_version = AppVersionEntry::try_from( record ).ok()?;

                if app_version.for_app != input.for_app {
                    return None
                }

                Some(
                    Entity {
                        id: origin.into_action_hash()?,
                        address: hash_entry( app_version.clone() ).ok()?,
                        action: addr,
                        ctype: app_version.get_type(),
                        content: app_version,
                    }
                )
            })
            .collect()
    )
}
