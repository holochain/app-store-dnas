mod constants;
mod host;

use hdk::prelude::*;
use hc_crud::{
    get_entity,
    Entity,
};
pub use portal::{
    LinkTypes,
    EntryTypes,
    EntryTypesUnit,

    HostEntry,

    AppResult, Response, EntityResponse,
    composition, catch,

    AppError,
    UserError,

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
    let host_targets = host::list_links_random( host::GetInput {
	dna: input.dna.to_owned(),
    } )?;

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

#[hdk_extern]
fn my_host_entries(_:()) -> ExternResult<Vec<HostEntry>> {
    Ok( host_entries()? )
}

pub fn host_entries() -> AppResult<Vec<HostEntry>> {
    query(ChainQueryFilter {
	sequence_range: ChainQueryFilterRange::Unbounded,
	entry_type: Some( EntryTypesUnit::Host.try_into()? ),
	entry_hashes: None,
	action_type: Some( ActionType::Create ),
	include_entries: true,
	order_descending: true,
    })?
	.into_iter()
	.map(|record| {
	    match record.entry {
		RecordEntry::Present(entry) => {
		    Ok( entry.try_into()? )
		},
		// Should be unreachable because of chain query filter settings
		_ => Err(UserError::StaticError("Expected entry; Chain query filter provided Create with no entry present"))?,
	    }
	})
	.collect()
}

pub fn latest_host_entry_for_dna(dna: &holo_hash::DnaHash) -> AppResult<Option<HostEntry>> {
    let dna_entries = host_entries()?
	.into_iter()
	.filter(|host_entry| host_entry.dna == *dna )
	.collect::<Vec<HostEntry>>();
    let host_entry = dna_entries.first();

    Ok( host_entry.map(|he| he.to_owned() ) )
}

fn handler_bridge_call(input: BridgeCallInput) -> AppResult<rmpv::Value> {
    let agent_info = agent_info()?;

    // Need to add a check here for this agent's registered zome functions
    match latest_host_entry_for_dna( &input.dna )? {
	Some(host_entry) => {
	    match host_entry.capabilities.access {
		CapAccess::Unrestricted => (),
		_ => return Err(UserError::CustomError(format!("Access is conditional for DNA {}, but only Unrestricted is supported at this time", input.dna )))?,
	    }

	    match host_entry.capabilities.functions {
		GrantedFunctions::Listed( granted_functions ) => {
		    if let None = granted_functions
			.into_iter()
			.find(|(zome, function)| {
			    return *zome == input.zome.clone().into()
				&& *function == input.function.clone().into()
			})
		    {
			Err(UserError::CustomError(format!("No capability granted for DNA zome/function {}/{}", input.zome, input.function )))?;
		    }
		}
		_ => (),
	    }
	},
	None => {
	    return Err(UserError::CustomError(format!("No host record for DNA {}", input.dna )))?;
	},
    };

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
fn bridge_call(input: BridgeCallInput) -> ExternResult<Response<rmpv::Value>> {
    let result = catch!( handler_bridge_call( input ) );

     Ok(composition( result, VALUE_MD ))
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



#[derive(Debug, Deserialize)]
pub struct CustomRemoteCallInput {
    host: AgentPubKey,
    call: RemoteCallInput,
}

fn handler_custom_remote_call(input: CustomRemoteCallInput) -> AppResult<Response<rmpv::Value>> {
    let call_details = BridgeCallDetails {
	dna: input.call.dna,
	zome: input.call.zome,
	function: input.call.function,
	payload: input.call.payload,
    };

    let response = call_remote(
	input.host,
	"portal_api",
	"bridge_call".into(),
	None,
	call_details,
    )?;

    let result = hc_utils::zome_call_response_as_result( response )?;

    Ok( result.decode()? )
}

#[hdk_extern]
fn custom_remote_call(input: CustomRemoteCallInput) -> ExternResult<Response<rmpv::Value>> {
    Ok( handler_custom_remote_call( input )? )
}
