use hdi::prelude::*;
use crate::{
    CommonFields,

    HostEntry,

    EntryTypes,
    // LinkTypes,
};



#[hdk_extern]
fn validate(op: Op) -> ExternResult<ValidateCallbackResult> {
    match op.clone() {
	// When any entry is being posted to the DHT
	Op::StoreEntry( store_entry ) => {
	    if let Some( entry_type ) = hc_utils::store_entry_deconstruct( &store_entry )? {
		debug!("Op::{} => Running validation for: {:?}", op.action_type(), entry_type );
		return match entry_type {
		    EntryTypes::Host(content) => validate_host_create( &op, content ),
		};
	    } else {
		if let Entry::CapGrant(_) = store_entry.entry {
		    return Ok(ValidateCallbackResult::Valid);
		}
	    }
	},

	// When the created entry is an update
	Op::RegisterUpdate( register_update ) => {
	    if let Some( entry_type ) = hc_utils::register_update_deconstruct( &register_update )? {
		debug!("Op::{} => Running validation for: {:?}", op.action_type(), entry_type );
		return match entry_type {
		    EntryTypes::Host(content) => {
			let original_entry : HostEntry = register_update.original_entry.unwrap().try_into()?;
			validate_host_update( &op, content, original_entry )
		    },
		};
	    }
	},

	// When deleting an entry creation
	Op::RegisterDelete( register_delete ) => {
	    if let Some( entry_type ) = hc_utils::register_delete_deconstruct( &register_delete )? {
		debug!("Op::{} => Running validation for: {:?}", op.action_type(), entry_type );
		return match entry_type {
		    EntryTypes::Host(original_entry) => validate_host_delete( &op, original_entry ),
		};
	    }
	},

	// Ignore the rest
	//  - StoreRecord
	//  - RegisterAgentActivity
	//  - RegisterCreateLink
	//  - RegisterDeleteLink
	_ => {
	    debug!("Op::{} => No validation", op.action_type() );
	    return Ok(ValidateCallbackResult::Valid);
	}
    }

    debug!("Op::{} => Validation fall-through: {:#?}", op.action_type(), op );
    Ok(ValidateCallbackResult::Valid)
}



fn validate_common_fields_create<'a, T>(op: &Op, entry: &'a T) -> ExternResult<ValidateCallbackResult>
where
    T: CommonFields<'a>,
{
    if entry.author() != op.author() {
	Ok(ValidateCallbackResult::Invalid(format!("Entry author does not match Action author: {} != {}", entry.author(), op.author() )))
    }
    else {
	Ok(ValidateCallbackResult::Valid)
    }
}

fn validate_common_fields_update<'a, T>(op: &Op, entry: &'a T, prev_entry: &'a T) -> ExternResult<ValidateCallbackResult>
where
    T: CommonFields<'a>,
{
    if let Err(error) = validate_common_fields_create(op, entry) {
	Err(error)?
    }

    if prev_entry.author() != op.author() {
	return Ok(ValidateCallbackResult::Invalid(format!("Previous entry author does not match Action author: {} != {}", prev_entry.author(), op.author() )));
    }
    else if entry.author() != prev_entry.author()  {
	return Ok(ValidateCallbackResult::Invalid(format!("Cannot change app author: {} => {}", prev_entry.author(), entry.author() )));
    }
    else {
	Ok(ValidateCallbackResult::Valid)
    }
}


//
// Host
//
fn validate_host_create(op: &Op, entry: HostEntry) -> ExternResult<ValidateCallbackResult> {
    if let Err(error) = validate_common_fields_create(op, &entry) {
	Err(error)?
    }

    Ok(ValidateCallbackResult::Valid)
}

fn validate_host_update(op: &Op, entry: HostEntry, prev_entry: HostEntry) -> ExternResult<ValidateCallbackResult> {
    if let Err(error) = validate_common_fields_update(op, &entry, &prev_entry) {
	Err(error)?
    }

    Ok(ValidateCallbackResult::Valid)
}

fn validate_host_delete(_op: &Op, _entry: HostEntry) -> ExternResult<ValidateCallbackResult> {
    Ok(ValidateCallbackResult::Valid)
}
