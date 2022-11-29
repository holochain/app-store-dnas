use hdi::prelude::*;
use crate::{
    CommonFields,

    PublisherEntry,
    AppEntry,

    EntryTypes,
    LinkTypes,
};


enum RecordEntryRef<'a> {
    Present(&'a Entry),
    Hidden,
    NotApplicable,
    NotStored,
}

impl<'a> From<&'a RecordEntry> for RecordEntryRef<'a> {
    fn from(r: &'a RecordEntry) -> Self {
        match r {
            RecordEntry::Present(e) => RecordEntryRef::Present(e),
            RecordEntry::Hidden => RecordEntryRef::Hidden,
            RecordEntry::NotApplicable => RecordEntryRef::NotApplicable,
            RecordEntry::NotStored => RecordEntryRef::NotStored,
        }
    }
}


#[hdk_extern]
fn validate(op: Op) -> ExternResult<ValidateCallbackResult> {
    debug!("Processing Op: {:?}", op );

    if let Op::StoreRecord(StoreRecord { record }) = op.clone() {
	let entry_ref : RecordEntryRef = (&record.entry).into();

	if let Action::Create(Create {
	    entry_type,
	    entry_hash,
	    ..
	}) = record.action() {
	    debug!("Captured Action::Create entry_hash: {:?}", entry_hash );
	    debug!("Captured Action::Create entry_type: {:?}", entry_type );

	    if let RecordEntryRef::Present(entry) = entry_ref {
		debug!("Captured Action::Create entry: {:?}", entry );
		if let hdi::prelude::Entry::CapGrant(ZomeCallCapGrant { .. }) = entry {
		    debug!("Allowing Action::Create CapGrant");
		    return Ok(ValidateCallbackResult::Valid);
		}
	    }
	}
    }

    if let Op::StoreEntry(StoreEntry { action, entry }) = op.clone() {
	debug!("Captured StoreEntry Action: {:?}", action );
	debug!("Captured StoreEntry Entry: {:?}", entry );

	if let hdi::prelude::Entry::CapGrant(ZomeCallCapGrant { .. }) = entry {
	    debug!("Allowing Action::Create CapGrant");
	    return Ok(ValidateCallbackResult::Valid);
	}
    }

    match op.to_type::<EntryTypes, LinkTypes>()? {
	OpType::StoreRecord( op_record ) => {
	    match op_record {
		OpRecord::CreateEntry { entry_type, .. } => {
		    debug!("Running create validation for: {:?}", entry_type );
		    match entry_type {
			EntryTypes::Publisher(entry) => validate_publisher_create( &op, entry ),
			EntryTypes::App(entry) => validate_app_create( &op, entry ),
		    }
		},
		OpRecord::UpdateEntry { entry_type, original_entry_hash, .. } => {
		    debug!("Running create validation for: {:?}", entry_type );
		    match entry_type {
			EntryTypes::Publisher(entry) => validate_publisher_update( &op, entry, &original_entry_hash ),
			EntryTypes::App(entry) => validate_app_update( &op, entry, &original_entry_hash ),
		    }
		},
		_ => {
		    debug!("Ignoring OpRecord: {:?}", op_record );
		    Ok(ValidateCallbackResult::Valid)
		},
	    }
	},
	_ => {
	    debug!("Ignoring Op event");
	    Ok(ValidateCallbackResult::Valid)
	},
    }
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

fn validate_publisher_update(op: &Op, entry: PublisherEntry, original_entry_hash: &EntryHash) -> ExternResult<ValidateCallbackResult> {
    let prev_entry : PublisherEntry = must_get_entry( original_entry_hash.to_owned() )?.try_into()?;

    if let Err(error) = validate_common_fields_update(op, &entry, &prev_entry) {
	Err(error)?
    }

    if prev_entry.deprecation.is_some() {
	return Ok(ValidateCallbackResult::Invalid("Cannot update deprecated app".to_string()));
    }

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

fn validate_app_update(op: &Op, entry: AppEntry, original_entry_hash: &EntryHash) -> ExternResult<ValidateCallbackResult> {
    let prev_entry : AppEntry = must_get_entry( original_entry_hash.to_owned() )?.try_into()?;

    if let Err(error) = validate_common_fields_update(op, &entry, &prev_entry) {
	Err(error)?
    }

    if prev_entry.deprecation.is_some() {
	return Ok(ValidateCallbackResult::Invalid("Cannot update deprecated app".to_string()));
    }

    Ok(ValidateCallbackResult::Valid)
}
