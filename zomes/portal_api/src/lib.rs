mod constants;
// mod host;

use rand::seq::SliceRandom;
use hdk::prelude::*;
pub use portal::{
    LinkTypes,
    EntryTypes,

    // HostEntry,

    // GetEntityInput, EntityId,
    AppResult, Response, EntityResponse,
    composition, catch,

    AppError,
};
pub use constants::{
    ENTITY_MD,
    VALUE_MD,

    // ANCHOR_AGENTS,
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



#[derive(Debug, Serialize, Deserialize)]
pub struct RemoteCallDetails<Z,F,I>
where
    Z: Into<ZomeName>,
    F: Into<FunctionName>,
    I: Serialize + core::fmt::Debug,
{
    pub agents: Vec<AgentPubKey>,
    pub zome: Z,
    pub function: F,
    pub payload: I,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BridgeCallDetails<Z,F,P>
where
    Z: Into<ZomeName>,
    F: Into<FunctionName>,
    P: Serialize + core::fmt::Debug,
{
    pub zome: Z,
    pub function: F,
    pub payload: P,
}

type Payload = rmpv::Value;
type RemoteCallInput = RemoteCallDetails<String, String, Payload>;
type BridgeCallInput = BridgeCallDetails<String, String, Payload>;



fn handler_remote_call(input: RemoteCallInput) -> AppResult<rmpv::Value> {
    let agent = input.agents.choose(&mut rand::thread_rng())
	.ok_or("There is no Host for this call".to_string())?;

    let response = call_remote(
	agent.clone(),
	"portal_api",
	"bridge_call".into(),
	None,
	BridgeCallDetails {
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
    let response = call(
	CallTargetCell::Local,
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



// #[hdk_extern]
// fn register_host(input: host::CreateInput) -> ExternResult<EntityResponse<HostEntry>> {
//     let entity = catch!( host::create( input ) );

//     Ok(composition( entity, ENTITY_MD ))
// }
