pub mod publisher;
pub mod app;
pub mod app_version;
pub mod group;
pub mod viewpoint;

pub use hdk_extensions::hdk;
pub use appstore::{
    LinkTypes,
    ALL_APPS_ANCHOR,
    appstore_types,
    hc_crud,
    hdi_extensions,
};
use hdk::prelude::*;
use appstore_types::*;
use apphub_sdk::{
    AppEntryInput as AppHubAppEntryInput,
    WebAppEntryInput,
    WebAppPackageEntryInput,
    WebAppPackageVersionEntryInput,
    apphub_types::{
        UiEntry,
        AppEntry as AppHubAppEntry,
        WebAppEntry,
        WebAppPackageEntry,
        WebAppPackageVersionEntry,
        mere_memory_types::{
            MemoryEntry,
            MemoryBlockEntry,
        },
    },
};


#[derive(Debug, Serialize, Deserialize)]
pub struct GetForAgentInput {
    pub for_agent: AgentPubKey,
}


#[derive(Debug, Serialize, Deserialize)]
pub struct GetForGroupInput {
    pub for_group: ActionHash,
}


#[derive(Debug, Serialize, Deserialize)]
pub struct GetForPublisherInput {
    pub for_publisher: EntityId,
}


#[derive(Debug, Serialize, Deserialize)]
pub struct GetForAppInput {
    pub for_app: EntityId,
}


#[hdk_extern]
fn init(_: ()) -> ExternResult<InitCallbackResult> {
    Ok(InitCallbackResult::Pass)
}


/// Get [`AgentInfo`] for this cell
#[hdk_extern]
pub fn whoami(_: ()) -> ExternResult<AgentInfo> {
    agent_info()
}



/// Calculate the [`EntryHash`] for the AppHub entry type [`WebAppPackageEntry`]
#[hdk_extern]
pub fn hash_webapp_package_entry(input: WebAppPackageEntryInput) -> ExternResult<EntryHash> {
    // debug!("WebAppPackageEntry: {:#?}", input );
    hash_entry( WebAppPackageEntry::from( input ) )
}


/// Calculate the [`EntryHash`] for the AppHub entry type [`WebAppPackageVersionEntry`]
#[hdk_extern]
pub fn hash_webapp_package_version_entry(input: WebAppPackageVersionEntryInput) -> ExternResult<EntryHash> {
    // debug!("WebAppPackageVersionEntry: {:#?}", input );
    hash_entry( WebAppPackageVersionEntry::from( input ) )
}


/// Calculate the [`EntryHash`] for the AppHub entry type [`WebAppEntry`]
#[hdk_extern]
pub fn hash_webapp_entry(input: WebAppEntryInput) -> ExternResult<EntryHash> {
    // debug!("WebAppEntry: {:#?}", input );
    hash_entry( WebAppEntry::from( input ) )
}


/// Calculate the [`EntryHash`] for the AppHub entry type [`UiEntry`]
#[hdk_extern]
pub fn hash_ui_entry(input: UiEntry) -> ExternResult<EntryHash> {
    // debug!("UiEntry: {:#?}", input );
    hash_entry( input )
}


/// Calculate the [`EntryHash`] for the AppHub entry type [`apphub_sdk::apphub_types::AppEntry`]
#[hdk_extern]
pub fn hash_app_entry(input: AppHubAppEntryInput) -> ExternResult<EntryHash> {
    // debug!("AppHubAppEntry: {:#?}", input );
    hash_entry( AppHubAppEntry::from( input ) )
}


/// Calculate the [`EntryHash`] for the AppHub entry type [`MemoryEntry`]
#[hdk_extern]
pub fn hash_mere_memory_entry(input: MemoryEntry) -> ExternResult<EntryHash> {
    // debug!("MemoryEntry: {:#?}", input );
    hash_entry( input )
}


/// Calculate the [`EntryHash`] for the AppHub entry type [`MemoryBlockEntry`]
#[hdk_extern]
pub fn hash_mere_memory_block_entry(input: MemoryBlockEntry) -> ExternResult<EntryHash> {
    // debug!("MemoryBlockEntry: {:#?}", input );
    hash_entry( input )
}
