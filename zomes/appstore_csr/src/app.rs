use crate::{
    hdi_extensions,
    hdk,
    coop_content_sdk,
    GetForAgentInput,
    GetForGroupInput,
    GetForPublisherInput,
};

use std::collections::BTreeMap;
use hdi_extensions::{
    AnyLinkableHashTransformer,
};
use hdk::prelude::*;
use hdk_extensions::{
    agent_id,
    must_get,
};
use appstore::{
    LinkTypes,
    RmpvValue,
    HRL,
    DeprecationNotice,

    ALL_APPS_ANCHOR,
    AppEntry,

    hc_crud::{
        now, create_entity, update_entity,
        get_entities,
        EntityId, Entity, EntryModel,
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
    pub title: String,
    pub subtitle: String,
    pub description: String,
    pub icon: EntryHash,
    pub publisher: EntityId,
    pub apphub_hrl: HRL,
    pub apphub_hrl_hash: EntryHash,

    // optional
    pub editors_group_id: Option<(ActionHash, ActionHash)>,
    pub published_at: Option<u64>,
    pub last_updated: Option<u64>,
    pub metadata: Option<BTreeMap<String, RmpvValue>>,
}

#[hdk_extern]
pub fn create_app(input: CreateInput) -> ExternResult<Entity<AppEntry>> {
    debug!("Creating App: {}", input.title );
    let default_now = now()?;

    // Get the latest group ref based on the parent publisher
    let editors_group_id = match input.editors_group_id {
        None => {
            let publisher_entry = crate::publisher::get_publisher(GetEntityInput {
                id: input.publisher.clone(),
            })?.content;
            let editors_group_id = publisher_entry.group_ref().0;
            (
                editors_group_id.clone(),
                crate::group::get_group( editors_group_id )?.action,
            )
        },
        Some(editors_group_id) => editors_group_id,
    };

    let app = AppEntry {
	title: input.title,
	subtitle: input.subtitle,
	description: input.description,
	icon: input.icon,
	publisher: input.publisher.clone(),
	apphub_hrl: input.apphub_hrl,
	apphub_hrl_hash: input.apphub_hrl_hash,
	editors_group_id: editors_group_id,

	published_at: input.published_at
	    .unwrap_or( default_now ),
	last_updated: input.last_updated
	    .unwrap_or( default_now ),

	metadata: input.metadata
	    .unwrap_or( BTreeMap::new() ),

	deprecation: None,
    };
    let entity = create_entity( &app )?;

    // Link from group
    register_content_to_group!({
        entry: app,
        target: entity.id.clone(),
    })?;

    { // Path via All Apps
	entity.link_from(
            &ALL_APPS_ANCHOR.path_entry_hash()?,
            LinkTypes::AllAppsToApp,
            None
        )?;
    }

    Ok( entity )
}


#[hdk_extern]
pub fn get_app(input: GetEntityInput) -> ExternResult<Entity<AppEntry>> {
    debug!("Get app: {}", input.id );
    let app_origin : AppEntry = must_get( &input.id )?.try_into()?;

    let latest_addr = get_group_content_latest!({
        group_id: app_origin.group_ref().0,
        content_id: input.id.clone().into(),
    })?;

    let app : AppEntry = must_get( &latest_addr )?.try_into()?;

    Ok(
	Entity {
            id: input.id,
            address: hash_entry( app.clone() )?,
            action: latest_addr,
            ctype: app.get_type(),
            content: app,
        }
    )
}


#[derive(Debug, Deserialize, Clone)]
pub struct UpdateProperties {
    pub title: Option<String>,
    pub subtitle: Option<String>,
    pub description: Option<String>,
    pub icon: Option<EntryHash>,
    pub apphub_hrl: Option<HRL>,
    pub apphub_hrl_hash: Option<EntryHash>,
    pub published_at: Option<u64>,
    pub last_updated: Option<u64>,
    pub metadata: Option<BTreeMap<String, RmpvValue>>,
}
pub type UpdateInput = UpdateEntityInput<UpdateProperties>;

#[hdk_extern]
pub fn update_app(input: UpdateInput) -> ExternResult<Entity<AppEntry>> {
    debug!("Updating App: {}", input.base );
    let props = input.properties.clone();

    let entity = update_entity(
	&input.base,
	|mut current : AppEntry, _| {
	    current.title = props.title
		.unwrap_or( current.title );
	    current.subtitle = props.subtitle
		.unwrap_or( current.subtitle );
	    current.description = props.description
		.unwrap_or( current.description );
	    current.apphub_hrl = props.apphub_hrl
		.unwrap_or( current.apphub_hrl );
	    current.apphub_hrl_hash = props.apphub_hrl_hash
		.unwrap_or( current.apphub_hrl_hash );

            // Automatically update the group ref to latest
            update_app_editors_group_ref( &mut current )?;

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

    register_content_update_to_group!({
        entry: entity.content.clone(),
        target: entity.action.clone(),
    })?;

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
            // Automatically update the group ref to latest
            update_app_editors_group_ref( &mut current )?;

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
    debug!("Undeprecating App: {}", input.base );
    let entity = update_entity(
	&input.base,
	|mut current : AppEntry, _| {
            // Automatically update the group ref to latest
            update_app_editors_group_ref( &mut current )?;

	    current.deprecation = None;

	    Ok( current )
	})?;

    Ok( entity )
}


#[hdk_extern]
pub fn get_apps_for_group(input: GetForGroupInput) -> ExternResult<Vec<Entity<AppEntry>>> {
    Ok(
        get_all_group_content_latest!({
            group_id: input.for_group,
        })?.into_iter()
            .filter_map(|(origin, latest)| {
                let addr = latest.into_action_hash()?;
                let record = must_get( &addr ).ok()?;
                let app = AppEntry::try_from( record ).ok()?;
                Some(
                    Entity {
                        id: origin.into_action_hash()?,
                        address: hash_entry( app.clone() ).ok()?,
                        action: addr,
                        ctype: app.get_type(),
                        content: app,
                    }
                )
            })
            .collect()
    )
}


/// Get all Apps for a given [`AgentPubKey`]
#[hdk_extern]
pub fn get_apps_for_agent(input: GetForAgentInput) -> ExternResult<Vec<Entity<AppEntry>>> {
    // Get groups that the agent belongs to
    let group_links = get_links(
        GetLinksInputBuilder::try_new(
            input.for_agent,
            LinkTypes::AgentToGroup,
        )?.build()
    )?;

    // For each group, get the apps that belong to that group
    let collection = group_links.iter()
        .filter_map( |link| {
            let group_id = link.target.must_be_action_hash().ok()?;

            debug!("Get all app content for group: {}", group_id );
            Some( get_apps_for_group( GetForGroupInput {
                for_group: group_id,
            }).ok()? )
        })
        .flatten()
        .collect();

    Ok( collection )
}


/// Get Apps that the current cell agent maintains
#[hdk_extern]
pub fn get_my_apps(_:()) -> ExternResult<Vec<Entity<AppEntry>>> {
    get_apps_for_agent( GetForAgentInput {
	for_agent: agent_id()?,
    })
}


/// Get Apps that belong to the given Publisher ID
#[hdk_extern]
pub fn get_apps_for_publisher(input: GetForPublisherInput) -> ExternResult<Vec<Entity<AppEntry>>> {
    let publisher_entry = crate::publisher::get_publisher(GetEntityInput {
        id: input.for_publisher,
    })?.content;

    // TODO: what happens if multiple publishers used this group?  Do we care?  Will it just be by
    // convention that you shouldn't reuse groups
    Ok(
        get_all_group_content_latest!({
            group_id: publisher_entry.group_ref().0,
        })?.into_iter()
            .filter_map(|(origin, latest)| {
                let addr = latest.into_action_hash()?;
                let record = must_get( &addr ).ok()?;
                let app = AppEntry::try_from( record ).ok()?;
                Some(
                    Entity {
                        id: origin.into_action_hash()?,
                        address: hash_entry( app.clone() ).ok()?,
                        action: addr,
                        ctype: app.get_type(),
                        content: app,
                    }
                )
            })
            .collect()
    )
}


/// Get all Apps
#[hdk_extern]
pub fn get_all_apps(_: ()) -> ExternResult<Vec<Entity<AppEntry>>> {
    let collection = get_entities(
        &ALL_APPS_ANCHOR.path_entry_hash()?,
        LinkTypes::AllAppsToApp,
        None
    )?;
    let collection = collection.into_iter()
	.filter(|entity : &Entity<AppEntry>| {
	    entity.content.deprecation.is_none()
	})
	.collect();

    Ok( collection )
}


/// Update the group ref to latest
fn update_app_editors_group_ref(app: &mut AppEntry) -> ExternResult<()> {
    let group_entity = crate::group::get_group( app.group_ref().0 )?;

    app.editors_group_id = (
        app.group_ref().0, group_entity.action
    );

    Ok(())
}
