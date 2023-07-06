use hdk::prelude::*;
use hdk::hash_path::path::{ Component };

pub type UtilResult<T> = Result<T, String>;

pub fn guest_err( message: String ) -> WasmError {
    wasm_error!(WasmErrorInner::Guest(message))
}

pub fn store_entry_deconstruct<ET>( store_entry: &StoreEntry ) -> ExternResult<Option<ET>>
where
    ET: EntryTypesHelper,
    WasmError: From<<ET as EntryTypesHelper>::Error>,
{
    Ok(match store_entry.action.hashed.content.entry_type() {
	EntryType::App(AppEntryDef {
	    zome_index,
	    entry_index,
	    ..
	}) => {
	    Some(
		ET::deserialize_from_type( *zome_index, *entry_index, &store_entry.entry )?
		    .ok_or( guest_err("No entry type matched for:".to_string()) )?
	    )
	},
	_ => None,
    })
}

pub fn register_update_deconstruct<ET>( register_update: &RegisterUpdate ) -> ExternResult<Option<ET>>
where
    ET: EntryTypesHelper,
    WasmError: From<<ET as EntryTypesHelper>::Error>,
{
    Ok(match register_update.original_action.entry_type() {
	EntryType::App(AppEntryDef {
	    zome_index,
	    entry_index,
	    visibility,
	}) => {
	    Some(match &register_update.new_entry {
		None => Err( guest_err(format!("New entry is None meaning visibility is Private: {:?}", visibility )) )?,
		Some(entry) => {
		    ET::deserialize_from_type( *zome_index, *entry_index, &entry )?
			.ok_or( guest_err("No entry type matched for:".to_string()) )?
		},
	    })
	},
	_ => None,
    })
}

pub fn register_delete_deconstruct<ET>( register_delete: &RegisterDelete ) -> ExternResult<Option<ET>>
where
    ET: EntryTypesHelper,
    WasmError: From<<ET as EntryTypesHelper>::Error>,
{
    Ok(match register_delete.original_action.entry_type() {
	EntryType::App(AppEntryDef {
	    zome_index,
	    entry_index,
	    visibility,
	}) => {
	    Some(match &register_delete.original_entry {
		None => Err( guest_err(format!("Original entry is None meaning visibility is Private: {:?}", visibility )) )?,
		Some(entry) => {
		    ET::deserialize_from_type( *zome_index, *entry_index, &entry )?
			.ok_or( guest_err("No entry type matched for:".to_string()) )?
		},
	    })
	},
	_ => None,
    })
}

pub fn path_base( base: &str ) -> (Path, EntryHash) {
    path( base, Vec::<String>::new() )
}
pub fn path<T>( base: &str, segments: T ) -> (Path, EntryHash)
where
    T: IntoIterator,
    T::Item: std::fmt::Display,
{
    let mut components : Vec<Component> = vec![];

    for seg in base.split(".") {
	let component = Component::from( format!("{}", seg ).as_bytes().to_vec() );
	components.push( component );
    }

    for seg in segments {
	let component = Component::from( format!("{}", seg ).as_bytes().to_vec() );
	components.push( component );
    }

    let path = Path::from( components );
    let hash = path.path_entry_hash().unwrap();

    ( path, hash )
}


pub fn agentpubkey () -> ExternResult<AgentPubKey> {
    Ok( agent_info()?.agent_initial_pubkey )
}

pub fn agentid () -> UtilResult<String> {
    Ok( format!("{}", agentpubkey()? ) )
}


pub fn zome_call_response_as_result(response: ZomeCallResponse) -> UtilResult<ExternIO> {
    Ok( match response {
	ZomeCallResponse::Ok(bytes)
	    => Ok(bytes),
	ZomeCallResponse::Unauthorized(zome_call_auth, cell_id, zome, func, agent)
	    => Err(format!("UnauthorizedError( {}, {}, {}, {}, {} )", zome_call_auth, cell_id, zome, func, agent )),
	ZomeCallResponse::NetworkError(message)
	    => Err(format!("NetworkError( {} )", message )),
	ZomeCallResponse::CountersigningSession(message)
	    => Err(format!("CountersigningSessionError( {} )", message )),
    }? )
}
