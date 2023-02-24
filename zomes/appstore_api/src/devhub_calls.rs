use hdk::prelude::*;
use appstore::{
    GetEntityInput,
};
use crate::{
    AppResult,
    Response,
    app::{
	get,
    },
};



#[derive(Debug, Serialize, Deserialize)]
pub struct GetWebHappPackageInput {
    pub name: String,
    pub happ_release_id: EntryHash,
    pub gui_release_id: EntryHash,
}

impl Into<rmpv::Value> for GetWebHappPackageInput {
    fn into(self) -> rmpv::Value {
	let serialized = rmp_serde::to_vec( &self ).unwrap();
	rmp_serde::from_slice( &serialized ).unwrap()
    }
}


pub fn get_webhapp_package(input: GetEntityInput) -> AppResult<Vec<u8>> {
    let entity = get( input )?;

    debug!("Call portal->remote_call# happ_library.get_webhapp_package()");
    let response = call(
	CallTargetCell::OtherRole("portal".into()),
	"portal_api",
	"remote_call".into(),
	None, // CapSecret
	portal_types::RemoteCallInput {
	    dna: entity.content.devhub_address.dna,
	    zome: "happ_library".to_string(),
	    function: "get_webhapp_package".to_string(),
	    payload: GetWebHappPackageInput {
		name: entity.content.name,
		happ_release_id: entity.content.devhub_address.happ,
		gui_release_id: entity.content.devhub_address.gui,
	    }.into(),
	}
    )?;
    let result = hc_utils::zome_call_response_as_result( response )?;
    let essence_resp : Response<Vec<u8>> = result.decode()?;

    Ok( essence_resp.as_result()? )
}
