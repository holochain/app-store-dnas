mod constants;
mod host;

use rand::seq::SliceRandom;
use hdk::prelude::*;
use hc_crud::{
    get_entity,
    Entity,
};
pub use portal::{
    LinkTypes,
    EntryTypes,

    HostEntry,

    AppResult, Response, EntityResponse,
    composition, catch,

    AppError,

    RemoteCallDetails,
    BridgeCallDetails,
    Payload,
    RemoteCallInput,
    BridgeCallInput,
};
pub use constants::{
    ENTITY_MD,
    VALUE_MD,

    ANCHOR_HOSTS,
};



#[hdk_extern]
fn init(_: ()) -> ExternResult<InitCallbackResult> {
    let mut anonymous_caps = BTreeSet::new();
    let zome_info = zome_info()?;

    anonymous_caps.insert( (zome_info.name, FunctionName::new("bridge_call")) );

    create_cap_grant( CapGrantEntry {
	tag: String::from("Public Functions"),
	access: CapAccess::Unrestricted,
	functions: anonymous_caps,
    })?;

    Ok(InitCallbackResult::Pass)
}


#[hdk_extern]
fn whoami(_: ()) -> ExternResult<Response<AgentInfo>> {
    Ok(composition( agent_info()?, VALUE_MD ))
}



fn handler_remote_call(input: RemoteCallInput) -> AppResult<rmpv::Value> {
    let (_, pathhash ) = hc_utils::path( ANCHOR_HOSTS, vec![
	&input.dna.to_string(),
	&input.zome,
	&input.function,
    ]);
    let links = get_links( pathhash, LinkTypes::Host, None )?;
    let entity_id : ActionHash = links.choose(&mut rand::thread_rng())
	.ok_or("There is no Host for this call".to_string())?
	.target.clone().into();
    let host_entry : Entity<HostEntry> = get_entity( &entity_id )?;

    let response = call_remote(
	host_entry.content.author,
	"portal_api",
	"bridge_call".into(),
	None,
	BridgeCallDetails {
	    dna: input.dna,
	    zome: input.zome,
	    function: input.function,
	    payload: input.payload,
	}
    )?;
    let result = hc_utils::zome_call_response_as_result( response )?;

    Ok( result.decode()? )
}

#[hdk_extern]
fn remote_call(input: RemoteCallInput) -> ExternResult<rmpv::Value> {
    let result = handler_remote_call( input )?;

    Ok( result )
}


fn handler_bridge_call(input: BridgeCallInput) -> AppResult<rmpv::Value> {
    let agent_info = agent_info()?;
    let cell_id = CellId::new( input.dna, agent_info.agent_initial_pubkey );

    let response = call(
	CallTargetCell::OtherCell( cell_id ),
	input.zome,
	input.function.into(),
	None,
	input.payload
    )?;
    let result = hc_utils::zome_call_response_as_result( response )?;

    Ok( result.decode()? )
}

#[hdk_extern]
fn bridge_call(input: BridgeCallInput) -> ExternResult<rmpv::Value> {
    Ok( handler_bridge_call( input )? )
}



#[hdk_extern]
fn register_host(input: host::CreateInput) -> ExternResult<EntityResponse<HostEntry>> {
    let entity = catch!( host::create( input ) );

    Ok(composition( entity, ENTITY_MD ))
}
