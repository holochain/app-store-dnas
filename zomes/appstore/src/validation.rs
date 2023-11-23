use hdi::prelude::*;
use crate::{
    CommonFields,

    PublisherEntry,
    AppEntry,
    ModeratorActionEntry,

    EntryTypes,
    // LinkTypes,
};
pub use mere_memory_types::{
    MemoryEntry,
};
use appstore_types::coop_content_sdk::{
    validate_group_auth,
};

const ICON_SIZE_LIMIT : u64 = 204_800;


#[hdk_extern]
fn validate(op: Op) -> ExternResult<ValidateCallbackResult> {
    match op.clone() {
	// When any entry is being posted to the DHT
	Op::StoreEntry( store_entry ) => {
	    if let Some( entry_type ) = hc_utils::store_entry_deconstruct( &store_entry )? {
		debug!("ActionType::{} => Op::StoreEntry: Running validation for: {:?}", op.action_type(), entry_type );
		return match entry_type {
		    EntryTypes::Publisher(content) => match op.action_type() {
			ActionType::Create => validate_publisher_create( &op, content ),
			_ => Ok(ValidateCallbackResult::Valid),
		    },
		    EntryTypes::App(content) => match op.action_type() {
			ActionType::Create => validate_app_create( &op, content ),
			_ => Ok(ValidateCallbackResult::Valid),
		    },
		    EntryTypes::ModeratorAction(content) => match op.action_type() {
			ActionType::Create => validate_moderator_action_create( &op, content ),
			_ => Ok(ValidateCallbackResult::Valid),
		    },
		    _ => Ok(ValidateCallbackResult::Valid),
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
		debug!("ActionType::{} => Op::RegisterUpdate: Running validation for: {:?}", op.action_type(), entry_type );
		return match entry_type {
		    EntryTypes::Publisher(content) => {
			let original_entry : PublisherEntry = register_update.original_entry.unwrap().try_into()?;
			validate_publisher_update( &op, content, original_entry )
		    },
		    EntryTypes::App(content) => {
			let original_entry : AppEntry = register_update.original_entry.unwrap().try_into()?;
			validate_app_update( &op, content, original_entry )
		    },
		    EntryTypes::ModeratorAction(content) => {
			let original_entry : ModeratorActionEntry = register_update.original_entry.unwrap().try_into()?;
			validate_moderator_action_update( &op, content, original_entry, register_update.update.hashed.content )
		    },
		    _ => Ok(ValidateCallbackResult::Valid),
		};
	    }
	},

	// When deleting an entry creation
	Op::RegisterDelete( register_delete ) => {
	    if let Some( entry_type ) = hc_utils::register_delete_deconstruct( &register_delete )? {
		debug!("ActionType::{} => Op::RegisterDelete: Running validation for: {:?}", op.action_type(), entry_type );
		return match entry_type {
		    EntryTypes::Publisher(original_entry) => validate_publisher_delete( &op, original_entry ),
		    EntryTypes::App(original_entry) => validate_app_delete( &op, original_entry ),
		    _ => Ok(ValidateCallbackResult::Valid),
		};
	    }
	},

	// Ignore the rest
	//  - StoreRecord
	//  - RegisterAgentActivity
	//  - RegisterCreateLink
	//  - RegisterDeleteLink
	_ => {
	    debug!("Op::{} => No validation handler", op.action_type() );
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
fn validate_common_publisher_fields(_op: &Op, entry: &PublisherEntry) -> ExternResult<ValidateCallbackResult> {
    let memory : MemoryEntry = must_get_entry( entry.icon.to_owned() )?.try_into()?;
    let icon_size = memory.uncompressed_size
        .unwrap_or( memory.memory_size );

    if icon_size > ICON_SIZE_LIMIT {
	Ok(ValidateCallbackResult::Invalid(format!("PublisherEntry icon cannot be larger than {}KB ({} bytes)", ICON_SIZE_LIMIT/1024, ICON_SIZE_LIMIT )))
    }
    else {
	Ok(ValidateCallbackResult::Valid)
    }
}

fn validate_publisher_create(op: &Op, entry: PublisherEntry) -> ExternResult<ValidateCallbackResult> {
    if let ValidateCallbackResult::Invalid(message) = validate_common_fields_create(op, &entry)? {
	return Ok(ValidateCallbackResult::Invalid(message));
    }

    if let ValidateCallbackResult::Invalid(message) = validate_common_publisher_fields(op, &entry)? {
	return Ok(ValidateCallbackResult::Invalid(message));
    }

    Ok(ValidateCallbackResult::Valid)
}

fn validate_publisher_update(op: &Op, entry: PublisherEntry, prev_entry: PublisherEntry) -> ExternResult<ValidateCallbackResult> {
    if let ValidateCallbackResult::Invalid(message) = validate_common_fields_update(op, &entry, &prev_entry)? {
	return Ok(ValidateCallbackResult::Invalid(message));
    }

    if prev_entry.deprecation.is_some() && entry.deprecation.is_some() {
	return Ok(ValidateCallbackResult::Invalid("Cannot update deprecated app".to_string()));
    }

    if let ValidateCallbackResult::Invalid(message) = validate_common_publisher_fields(op, &entry)? {
	return Ok(ValidateCallbackResult::Invalid(message));
    }

    Ok(ValidateCallbackResult::Valid)
}

fn validate_publisher_delete(_op: &Op, _entry: PublisherEntry) -> ExternResult<ValidateCallbackResult> {
    Ok(ValidateCallbackResult::Valid)
}



//
// App
//
fn validate_common_app_fields(_op: &Op, entry: &AppEntry) -> ExternResult<ValidateCallbackResult> {
    let memory : MemoryEntry = must_get_entry( entry.icon.to_owned() )?.try_into()?;
    let icon_size = memory.uncompressed_size
        .unwrap_or( memory.memory_size );

    if icon_size > ICON_SIZE_LIMIT {
	Ok(ValidateCallbackResult::Invalid(format!("AppEntry icon cannot be larger than {}KB ({} bytes)", ICON_SIZE_LIMIT/1024, ICON_SIZE_LIMIT )))
    }
    else {
	Ok(ValidateCallbackResult::Valid)
    }
}

fn validate_app_create(op: &Op, entry: AppEntry) -> ExternResult<ValidateCallbackResult> {
    if let ValidateCallbackResult::Invalid(message) = validate_common_fields_create(op, &entry)? {
	return Ok(ValidateCallbackResult::Invalid(message));
    }

    if let ValidateCallbackResult::Invalid(message) = validate_common_app_fields(op, &entry)? {
	return Ok(ValidateCallbackResult::Invalid(message));
    }

    Ok(ValidateCallbackResult::Valid)
}

fn validate_app_update(op: &Op, entry: AppEntry, prev_entry: AppEntry) -> ExternResult<ValidateCallbackResult> {
    if let ValidateCallbackResult::Invalid(message) = validate_common_fields_update(op, &entry, &prev_entry)? {
	return Ok(ValidateCallbackResult::Invalid(message));
    }

    if prev_entry.deprecation.is_some() && entry.deprecation.is_some() {
	return Ok(ValidateCallbackResult::Invalid("Cannot update deprecated app".to_string()));
    }

    if let ValidateCallbackResult::Invalid(message) = validate_common_app_fields(op, &entry)? {
	return Ok(ValidateCallbackResult::Invalid(message));
    }

    Ok(ValidateCallbackResult::Valid)
}

fn validate_app_delete(_op: &Op, _entry: AppEntry) -> ExternResult<ValidateCallbackResult> {
    Ok(ValidateCallbackResult::Valid)
}



//
// Moderator Action
//
fn validate_moderator_action_create(op: &Op, entry: ModeratorActionEntry) -> ExternResult<ValidateCallbackResult> {
    if &entry.author != op.author() {
        return Ok(ValidateCallbackResult::Invalid(format!("Entry author does not match Action author: {} != {}", entry.author, op.author() )));
    }

    Ok(ValidateCallbackResult::Valid)
}

fn validate_moderator_action_update(op: &Op, entry: ModeratorActionEntry, _prev_entry: ModeratorActionEntry, update: Update) -> ExternResult<ValidateCallbackResult> {
    if &entry.author != op.author() {
        return Ok(ValidateCallbackResult::Invalid(format!("Entry author does not match Action author: {} != {}", entry.author, op.author() )));
    }

    if let Err(message) = validate_group_auth( &entry, update ) {
        return Ok(ValidateCallbackResult::Invalid(message));
    }

    Ok(ValidateCallbackResult::Valid)
}
