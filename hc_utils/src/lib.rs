use hdk::prelude::*;
use hdk::hash_path::path::{ Component };

pub type UtilResult<T> = Result<T, String>;

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


pub fn agentid () -> UtilResult<String> {
    Ok( format!("{}", agent_info()?.agent_initial_pubkey ) )
}


pub fn zome_call_response_as_result(response: ZomeCallResponse) -> UtilResult<zome_io::ExternIO> {
    Ok( match response {
	ZomeCallResponse::Ok(bytes)
	    => Ok(bytes),
	ZomeCallResponse::Unauthorized(cell_id, zome, func, agent)
	    => Err(format!("UnauthorizedError( {}, {}, {}, {} )", cell_id, zome, func, agent )),
	ZomeCallResponse::NetworkError(message)
	    => Err(format!("NetworkError( {} )", message )),
	ZomeCallResponse::CountersigningSession(message)
	    => Err(format!("CountersigningSessionError( {} )", message )),
    }? )
}
