use crate::{
    hdi_extensions,
    hdk,
    coop_content_sdk,
    GetForAgentInput,
    GetForGroupInput,
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
    WebAddress,
    DeprecationNotice,

    PublisherEntry,

    hc_crud::{
        now, create_entity, update_entity,
        Entity, EntryModel,
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
    pub name: String,
    pub location: String,
    pub website: WebAddress,
    pub editors_group_id: (ActionHash, ActionHash),

    // optional
    pub description: Option<String>,
    pub email: Option<String>,
    pub icon: Option<EntryHash>,

    pub published_at: Option<u64>,
    pub last_updated: Option<u64>,
    pub metadata: Option<BTreeMap<String, RmpvValue>>,
}


#[hdk_extern]
pub fn create_publisher(input: CreateInput) -> ExternResult<Entity<PublisherEntry>> {
    debug!("Creating Publisher: {}", input.name );
    let default_now = now()?;

    let publisher = PublisherEntry {
	name: input.name,
	description: input.description,
	location: input.location,
	website: input.website,

	editors_group_id: input.editors_group_id,

	published_at: input.published_at
	    .unwrap_or( default_now ),
	last_updated: input.last_updated
	    .unwrap_or( default_now ),

	metadata: input.metadata
	    .unwrap_or( BTreeMap::new() ),

	email: input.email,
	icon: input.icon,
	deprecation: None,
    };
    let entity = create_entity( &publisher )?;

    // Link from group
    register_content_to_group!({
        entry: publisher,
        target: entity.id.clone(),
    })?;

    Ok( entity )
}


#[hdk_extern]
pub fn get_publisher(input: GetEntityInput) -> ExternResult<Entity<PublisherEntry>> {
    debug!("Get publisher: {}", input.id );
    let publisher_origin : PublisherEntry = must_get( &input.id )?.try_into()?;

    let latest_addr = get_group_content_latest!({
        group_id: publisher_origin.group_ref().0,
        content_id: input.id.clone().into(),
    })?;

    let publisher : PublisherEntry = must_get( &latest_addr )?.try_into()?;

    Ok(
	Entity {
            id: input.id,
            address: hash_entry( publisher.clone() )?,
            action: latest_addr,
            ctype: publisher.get_type(),
            content: publisher,
        }
    )
}


#[derive(Debug, Deserialize, Clone)]
pub struct UpdateProperties {
    pub name: Option<String>,
    pub description: Option<String>,
    pub location: Option<String>,
    pub website: Option<WebAddress>,
    pub icon: Option<EntryHash>,
    pub email: Option<String>,
    pub published_at: Option<u64>,
    pub last_updated: Option<u64>,
    pub metadata: Option<BTreeMap<String, RmpvValue>>,
}
pub type UpdateInput = UpdateEntityInput<UpdateProperties>;

#[hdk_extern]
pub fn update_publisher(input: UpdateInput) -> ExternResult<Entity<PublisherEntry>> {
    debug!("Updating Publisher: {}", input.base );
    let props = input.properties.clone();

    let entity = update_entity(
	&input.base,
	|mut current : PublisherEntry, _| {
	    current.name = props.name
		.unwrap_or( current.name );
	    current.description = props.description
		.or( current.description );
	    current.location = props.location
		.unwrap_or( current.location );
	    current.website = props.website
		.unwrap_or( current.website );

            // Automatically update the group ref to latest
            update_publisher_editors_group_ref( &mut current )?;

	    current.icon = props.icon
		.or( current.icon );
	    current.email = props.email
		.or( current.email );
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
pub fn deprecate_publisher(input: DeprecateInput) -> ExternResult<Entity<PublisherEntry>> {
    debug!("Deprecating publisher: {}", input.base );
    let entity = update_entity(
	&input.base,
	|mut current : PublisherEntry, _| {
            // Automatically update the group ref to latest
            update_publisher_editors_group_ref( &mut current )?;

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
pub fn undeprecate_publisher(input: UndeprecateInput) -> ExternResult<Entity<PublisherEntry>> {
    debug!("Undeprecating publisher: {}", input.base );
    let entity = update_entity(
	&input.base,
	|mut current : PublisherEntry, _| {
            // Automatically update the group ref to latest
            update_publisher_editors_group_ref( &mut current )?;

	    current.deprecation = None;

	    Ok( current )
	})?;

    Ok( entity )
}


#[hdk_extern]
pub fn get_publishers_for_group(input: GetForGroupInput) -> ExternResult<Vec<Entity<PublisherEntry>>> {
    Ok(
        get_all_group_content_latest!({
            group_id: input.for_group,
        })?.into_iter()
            .filter_map(|(origin, latest)| {
                let addr = latest.into_action_hash()?;
                let record = must_get( &addr ).ok()?;
                let publisher = PublisherEntry::try_from( record ).ok()?;
                Some(
                    Entity {
                        id: origin.into_action_hash()?,
                        address: hash_entry( publisher.clone() ).ok()?,
                        action: addr,
                        ctype: publisher.get_type(),
                        content: publisher,
                    }
                )
            })
            .collect()
    )
}


/// Get all Publishers for a given [`AgentPubKey`]
#[hdk_extern]
pub fn get_publishers_for_agent(input: GetForAgentInput) -> ExternResult<Vec<Entity<PublisherEntry>>> {
    // Get groups that the agent belongs to
    let group_links = get_links(
        GetLinksInputBuilder::try_new(
            input.for_agent,
            LinkTypes::AgentToGroup,
        )?.build()
    )?;

    // For each group, get the publishers that belong to that group
    let collection = group_links.iter()
        .filter_map( |link| {
            let group_id = link.target.must_be_action_hash().ok()?;

            debug!("Get all publisher content for group: {}", group_id );
            Some( get_publishers_for_group( GetForGroupInput {
                for_group: group_id,
            }).ok()? )
        })
        .flatten()
        .collect();

    Ok( collection )
}


/// Get Publishers that the current cell agent is a member of
#[hdk_extern]
pub fn get_my_publishers(_:()) -> ExternResult<Vec<Entity<PublisherEntry>>> {
    get_publishers_for_agent( GetForAgentInput {
	for_agent: agent_id()?,
    })
}


/// Update the group ref to latest
fn update_publisher_editors_group_ref(publisher: &mut PublisherEntry) -> ExternResult<()> {
    let group_entity = crate::group::get_group( publisher.group_ref().0 )?;

    publisher.editors_group_id = (
        publisher.group_ref().0, group_entity.action
    );

    Ok(())
}
