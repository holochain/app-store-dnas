use hdi::prelude::*;
use crate::{
    CommonFields,

    PublisherEntry,
    AppEntry,

    EntryTypes,
    // LinkTypes,
};


#[hdk_extern]
fn validate(op: Op) -> ExternResult<ValidateCallbackResult> {
    debug!("Op::{} => Validation", op.action_type() );

    match op.clone() {
	// When any entry is being posted to the DHT
	Op::StoreEntry( store_entry ) => {
	    if let Some( entry_type ) = hc_utils::store_entry_deconstruct( &store_entry )? {
		debug!("Running create validation for: {:?}", entry_type );
		return match entry_type {
		    EntryTypes::Publisher(content) => validate_publisher_create( &op, content ),
		    EntryTypes::App(content) => validate_app_create( &op, content ),
		};
	    }
	},

	// When the created entry is an update
	Op::RegisterUpdate( register_update ) => {
	    if let Some( entry_type ) = hc_utils::register_update_deconstruct( &register_update )? {
		debug!("Running update validation for: {:?}", entry_type );
		return match entry_type {
		    EntryTypes::Publisher(content) => {
			let original_entry : PublisherEntry = register_update.original_entry.unwrap().try_into()?;
			validate_publisher_update( &op, content, original_entry )
		    },
		    EntryTypes::App(content) => {
			let original_entry : AppEntry = register_update.original_entry.unwrap().try_into()?;
			validate_app_update( &op, content, original_entry )
		    },
		};
	    }
	},

	// When deleting an entry creation
	Op::RegisterDelete( register_delete ) => {
	    if let Some( entry_type ) = hc_utils::register_delete_deconstruct( &register_delete )? {
		debug!("Running delete validation for: {:?}", entry_type );
		return match entry_type {
		    EntryTypes::Publisher(original_entry) => validate_publisher_delete( &op, original_entry ),
		    EntryTypes::App(original_entry) => validate_app_delete( &op, original_entry ),
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
// Publisher
//
fn validate_publisher_create(op: &Op, entry: PublisherEntry) -> ExternResult<ValidateCallbackResult> {
    if let Err(error) = validate_common_fields_create(op, &entry) {
	Err(error)?
    }

    Ok(ValidateCallbackResult::Valid)
}

fn validate_publisher_update(op: &Op, entry: PublisherEntry, prev_entry: PublisherEntry) -> ExternResult<ValidateCallbackResult> {
    if let Err(error) = validate_common_fields_update(op, &entry, &prev_entry) {
	Err(error)?
    }

    if prev_entry.deprecation.is_some() {
	return Ok(ValidateCallbackResult::Invalid("Cannot update deprecated app".to_string()));
    }

    Ok(ValidateCallbackResult::Valid)
}

fn validate_publisher_delete(_op: &Op, _entry: PublisherEntry) -> ExternResult<ValidateCallbackResult> {
    Ok(ValidateCallbackResult::Valid)
}



//
// App
//
fn validate_app_create(op: &Op, entry: AppEntry) -> ExternResult<ValidateCallbackResult> {
    if let Err(error) = validate_common_fields_create(op, &entry) {
	Err(error)?
    }

    Ok(ValidateCallbackResult::Valid)
}

fn validate_app_update(op: &Op, entry: AppEntry, prev_entry: AppEntry) -> ExternResult<ValidateCallbackResult> {
    if let Err(error) = validate_common_fields_update(op, &entry, &prev_entry) {
	Err(error)?
    }

    if prev_entry.deprecation.is_some() {
	return Ok(ValidateCallbackResult::Invalid("Cannot update deprecated app".to_string()));
    }

    Ok(ValidateCallbackResult::Valid)
}

fn validate_app_delete(_op: &Op, _entry: AppEntry) -> ExternResult<ValidateCallbackResult> {
    Ok(ValidateCallbackResult::Valid)
}
