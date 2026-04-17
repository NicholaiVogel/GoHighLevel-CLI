//! Core library for the unofficial Go High Level CLI.
//!
//! Phase 0 intentionally contains only local scaffolding: config path resolution,
//! command metadata, endpoint manifest loading, and the stable error registry.

pub mod appointments;
pub mod audit;
pub mod auth;
pub mod calendars;
pub mod capabilities;
pub mod client;
pub mod config;
pub mod contacts;
pub mod context;
pub mod conversations;
pub mod credentials;
pub mod doctor;
pub mod endpoints;
pub mod errors;
pub mod idempotency;
pub mod locations;
pub mod metadata;
pub mod opportunities;
pub mod pipelines;
pub mod profiles;
pub mod redaction;
pub mod smoke;
pub mod surfaces;
pub mod users;

pub use appointments::{
    AppointmentCancelDryRun, AppointmentCancelOptions, AppointmentCreateDryRun,
    AppointmentCreateOptions, AppointmentCreateResult, AppointmentPreflightSummary,
    AppointmentStatus, AppointmentUpdateDryRun, AppointmentUpdateOptions, AppointmentUpdateStatus,
    AppointmentWriteResult, PreflightStatus, cancel_appointment, cancel_appointment_dry_run,
    create_appointment, create_appointment_dry_run, update_appointment, update_appointment_dry_run,
};
pub use audit::{
    AuditEntry, AuditEntryInput, AuditExportResult, AuditListOptions, AuditListResult,
    AuditResource, AuditResultSummary, AuditShowResult, AuditUpstreamSummary, append_audit_entry,
    audit_journal_path, export_audit_entries, list_audit_entries, parse_timestamp_filter,
    show_audit_entry,
};
pub use auth::{
    AuthStatus, LocalPitList, PitAddResult, PitRemoveResult, add_pit, auth_status, list_local_pits,
    remove_local_pit,
};
pub use calendars::{
    CalendarEventsDryRun, CalendarEventsOptions, CalendarEventsResult, CalendarFreeSlotsDryRun,
    CalendarFreeSlotsOptions, CalendarFreeSlotsResult, CalendarGetDryRun, CalendarGetResult,
    CalendarListDryRun, CalendarListOptions, CalendarListResult, calendar_events_dry_run,
    calendar_free_slots_dry_run, calendars_list_dry_run, get_calendar, get_calendar_dry_run,
    get_calendar_free_slots, list_calendar_events, list_calendars,
};
pub use capabilities::{
    CapabilityCheck, CapabilityConfidence, CapabilityReport, CapabilityState, capability_report,
    check_capability,
};
pub use client::{
    AuthClass, PitValidationResult, RawDeleteRequest, RawDeleteResponse, RawGetRequest,
    RawGetResponse, RawPostJsonRequest, RawPostJsonResponse, RawPutJsonRequest, RawPutJsonResponse,
    delete, post_json, put_json, raw_get, validate_pit,
};
pub use config::{CliConfig, ConfigDoctor, ConfigPaths, resolve_paths, resolve_paths_from_env};
pub use contacts::{
    ContactGetDryRun, ContactGetResult, ContactListDryRun, ContactListOptions, ContactListResult,
    ContactSearchDryRun, ContactSearchOptions, ContactSearchResult, contacts_list_dry_run,
    contacts_search_dry_run, get_contact, get_contact_dry_run, list_contacts, search_contacts,
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
pub use doctor::{
    CliDoctorInfo, DoctorBundleResult, DoctorReport, EndpointDoctorReport, doctor_api,
    doctor_endpoint, doctor_summary, write_support_bundle,
};
pub use endpoints::{EndpointCoverage, EndpointDefinition, EndpointManifest, bundled_manifest};
pub use errors::{ErrorDefinition, GhlError, Result, error_definitions, find_error_definition};
pub use idempotency::{
    IdempotencyCheck, IdempotencyCheckState, IdempotencyClearResult, IdempotencyListResult,
    IdempotencyPut, IdempotencyRecord, IdempotencyShowResult, IdempotencyStatus,
    check_idempotency_key, clear_idempotency_record, idempotency_store_path,
    list_idempotency_records, record_idempotency_key, scoped_idempotency_key,
    show_idempotency_record, stable_request_hash,
};
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
pub use users::{
    UserGetDryRun, UserGetResult, UserListDryRun, UserListOptions, UserListResult,
    UserSearchDryRun, UserSearchMode, UserSearchOptions, UserSearchResult, UserSortDirection,
    get_user, get_user_dry_run, list_users, search_users, users_list_dry_run, users_search_dry_run,
};
