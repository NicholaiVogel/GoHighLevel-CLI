//! Core library for the unofficial Go High Level CLI.
//!
//! Phase 0 intentionally contains only local scaffolding: config path resolution,
//! command metadata, endpoint manifest loading, and the stable error registry.

pub mod auth;
pub mod client;
pub mod config;
pub mod contacts;
pub mod context;
pub mod conversations;
pub mod credentials;
pub mod endpoints;
pub mod errors;
pub mod locations;
pub mod metadata;
pub mod opportunities;
pub mod pipelines;
pub mod profiles;
pub mod redaction;
pub mod smoke;
pub mod surfaces;

pub use auth::{
    AuthStatus, LocalPitList, PitAddResult, PitRemoveResult, add_pit, auth_status, list_local_pits,
    remove_local_pit,
};
pub use client::{
    AuthClass, PitValidationResult, RawGetRequest, RawGetResponse, RawPostJsonRequest,
    RawPostJsonResponse, post_json, raw_get, validate_pit,
};
pub use config::{CliConfig, ConfigDoctor, ConfigPaths, resolve_paths, resolve_paths_from_env};
pub use contacts::{
    ContactGetDryRun, ContactGetResult, ContactSearchDryRun, ContactSearchOptions,
    ContactSearchResult, contacts_search_dry_run, get_contact, get_contact_dry_run,
    search_contacts,
};
pub use context::{
    ContextSource, ResolvedContext, ResolvedContextValue, resolve_context,
    resolve_context_for_dry_run, resolve_context_for_profile,
};
pub use conversations::{
    ConversationGetDryRun, ConversationGetResult, ConversationMessagesDryRun,
    ConversationMessagesOptions, ConversationMessagesResult, ConversationSearchDryRun,
    ConversationSearchOptions, ConversationSearchResult, ConversationStatus,
    conversation_messages_dry_run, conversations_search_dry_run, get_conversation,
    get_conversation_dry_run, get_conversation_messages, search_conversations,
};
pub use credentials::{
    CredentialStore, RedactedCredential, credential_ref, load_credentials, save_credentials,
};
pub use endpoints::{EndpointCoverage, EndpointDefinition, EndpointManifest, bundled_manifest};
pub use errors::{ErrorDefinition, GhlError, Result, error_definitions, find_error_definition};
pub use locations::{
    LocationGetDryRun, LocationGetResult, LocationSearchDryRun, LocationSearchOptions,
    LocationSearchOrder, LocationSearchResult, get_location, get_location_dry_run, list_locations,
    locations_search_dry_run, search_locations,
};
pub use metadata::{CommandMetadata, CommandSchema, command_schema};
pub use opportunities::{
    OpportunityGetDryRun, OpportunityGetResult, OpportunitySearchDryRun, OpportunitySearchOptions,
    OpportunitySearchResult, OpportunityStatus, get_opportunity, get_opportunity_dry_run,
    opportunities_search_dry_run, search_opportunities,
};
pub use pipelines::{
    PipelineGetDryRun, PipelineGetResult, PipelineListDryRun, PipelineListResult, get_pipeline,
    get_pipeline_dry_run, list_pipelines, pipelines_list_dry_run,
};
pub use profiles::{
    Profile, ProfileCompanyResult, ProfileDefaultResult, ProfileList, ProfileLocationResult,
    ProfilePolicy, ProfilePolicyPatch, ProfilesFile, load_profiles, save_profiles,
    set_default_company, set_default_location, set_default_profile,
};
pub use smoke::{
    SmokeCheck, SmokeCheckStatus, SmokeRunMode, SmokeRunOptions, SmokeRunReport, SmokeSummary,
    smoke_run, smoke_run_dry_run,
};
pub use surfaces::Surface;
