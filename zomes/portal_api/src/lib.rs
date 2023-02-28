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
    ENTITY_COLLECTION_MD,
    VALUE_MD,

    ANCHOR_HOSTS,
};



#[hdk_extern]
fn init(_: ()) -> ExternResult<InitCallbackResult> {
    let mut anonymous_caps = BTreeSet::new();
    let zome_info = zome_info()?;

    anonymous_caps.insert( (zome_info.name.to_owned(), FunctionName::new("bridge_call")) );
    anonymous_caps.insert( (zome_info.name.to_owned(), FunctionName::new("pong")) );

    create_cap_grant( CapGrantEntry {
	tag: String::from("Public Functions"),
	access: CapAccess::Unrestricted,
	functions: GrantedFunctions::Listed( anonymous_caps ),
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
    let mut links = get_links( pathhash, LinkTypes::Host, None )?;

    if links.len() == 0 {
	return Err("There is no Host for this call".to_string())?;
    }

    links.shuffle(&mut rand::thread_rng());

    let host_targets : Vec<AnyLinkableHash> = links.into_iter()
	.map(|link| link.target)
	.collect();
    let call_details = BridgeCallDetails {
	dna: input.dna,
	zome: input.zome,
	function: input.function,
	payload: input.payload,
    };

    debug!("{} registered host(s)", host_targets.len() );
    for host_addr in host_targets {
	let host_entry : Entity<HostEntry> = get_entity( &host_addr.clone().into() )?;

	debug!("Attempting to remote call host: {}", host_entry.content.author );
	let response = call_remote(
	    host_entry.content.author,
	    "portal_api",
	    "bridge_call".into(),
	    None,
	    call_details.clone(),
	)?;

	if let ZomeCallResponse::NetworkError(message) = response.clone() {
	    if message.contains("agent is likely offline") {
		debug!("Host {} is offline, trying next host", host_addr );
		continue;
	    }
	}

	let result = hc_utils::zome_call_response_as_result( response )?;

	return Ok( result.decode()? );
    }

    Err("All hosts were unreachable".to_string())?
}

#[hdk_extern]
fn remote_call(input: RemoteCallInput) -> ExternResult<rmpv::Value> {
    let result = handler_remote_call( input )?;

    Ok( result )
}


fn handler_bridge_call(input: BridgeCallInput) -> AppResult<rmpv::Value> {
    let agent_info = agent_info()?;
    let cell_id = CellId::new( input.dna, agent_info.agent_initial_pubkey );

    debug!("Received remote call to bridge: {}::{}->{}", cell_id, input.zome, input.function );
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


fn handler_ping_call(host: AgentPubKey) -> AppResult<bool> {
    let response = call_remote(
	host,
	"portal_api",
	"pong".into(),
	None,
	(),
    )?;
    let result = hc_utils::zome_call_response_as_result( response )?;
    let _response : String = result.decode()?;

    Ok( true )
}

#[hdk_extern]
fn ping(host: AgentPubKey) -> ExternResult<Response<bool>> {
    debug!("Sending ping to host: {}", host );
    handler_ping_call( host )?;
    Ok(composition( true, VALUE_MD ))
}


#[hdk_extern]
fn pong(_: ()) -> ExternResult<String> {
    debug!("Responding with pong");
    Ok( String::from("pong") )
}



#[hdk_extern]
fn register_host(input: host::CreateInput) -> ExternResult<EntityResponse<HostEntry>> {
    let entity = catch!( host::create( input ) );

    Ok(composition( entity, ENTITY_MD ))
}



#[hdk_extern]
fn get_registered_hosts(input: host::GetInput) -> ExternResult<Response<Vec<Entity<HostEntry>>>> {
    let list = catch!( host::list( input ) );

    Ok(composition( list, ENTITY_COLLECTION_MD ))
}
