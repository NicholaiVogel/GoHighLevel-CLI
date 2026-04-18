use serde::{Deserialize, Serialize};
use serde_json::{Map, Value, json};

use crate::audit::{
    AuditEntryInput, AuditResource, AuditResultSummary, AuditUpstreamSummary, append_audit_entry,
};
use crate::client::{
    AuthClass, RawGetRequest, RawPostJsonRequest, RawPutJsonRequest, post_json, put_json, raw_get,
};
use crate::context::{ResolvedContext, resolve_context, resolve_context_for_dry_run};
use crate::errors::{GhlError, Result};
use crate::idempotency::{
    IdempotencyCheckState, IdempotencyPut, IdempotencyStatus, check_idempotency_key,
    record_idempotency_key, stable_request_hash,
};
use crate::profiles::load_profiles;
use crate::redaction::redact_json;
use crate::surfaces::Surface;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ContactGetResult {
    pub profile: String,
    pub context: ResolvedContext,
    pub location_id: String,
    pub contact_id: String,
    pub endpoint: String,
    pub url: String,
    pub status: u16,
    pub success: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub body_json: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub body_text: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ContactGetDryRun {
    pub method: &'static str,
    pub surface: &'static str,
    pub path: String,
    pub context: ResolvedContext,
    pub location_id: String,
    pub contact_id: String,
    pub auth_class: &'static str,
    pub network: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ContactSearchOptions {
    pub query: Option<String>,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub limit: u32,
    pub start_after_id: Option<String>,
    pub start_after: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ContactListOptions {
    pub limit: u32,
    pub start_after_id: Option<String>,
    pub start_after: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct ContactWriteFields {
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub name: Option<String>,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub address1: Option<String>,
    pub city: Option<String>,
    pub state: Option<String>,
    pub country: Option<String>,
    pub postal_code: Option<String>,
    pub website: Option<String>,
    pub timezone: Option<String>,
    pub company_name: Option<String>,
    pub source: Option<String>,
    pub tags: Vec<String>,
    pub assigned_to: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ContactCreateOptions {
    pub fields: ContactWriteFields,
    pub idempotency_key: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ContactUpdateOptions {
    pub contact_id: String,
    pub fields: ContactWriteFields,
    pub idempotency_key: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ContactDuplicatePreflight {
    pub status: String,
    pub checked: bool,
    pub email_checked: bool,
    pub phone_checked: bool,
    pub duplicate_count: usize,
    pub duplicate_contact_ids: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ContactWriteDryRun {
    pub method: &'static str,
    pub surface: &'static str,
    pub path: String,
    pub context: ResolvedContext,
    pub location_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub contact_id: Option<String>,
    pub request_body_json: Value,
    pub request_hash: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub idempotency_key: Option<String>,
    pub preflight: ContactDuplicatePreflight,
    pub audit_entry_id: String,
    pub auth_class: &'static str,
    pub network: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ContactWriteResult {
    pub profile: String,
    pub context: ResolvedContext,
    pub location_id: String,
    pub endpoint: String,
    pub url: String,
    pub status: u16,
    pub success: bool,
    pub replayed: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub contact_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub idempotency_key: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub idempotency_state: Option<String>,
    pub request_hash: String,
    pub preflight: ContactDuplicatePreflight,
    pub audit_entry_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub body_json: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub body_text: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ContactSearchResult {
    pub profile: String,
    pub context: ResolvedContext,
    pub location_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub query: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub email_filter: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub phone_filter: Option<String>,
    pub limit: u32,
    pub endpoint: String,
    pub url: String,
    pub status: u16,
    pub success: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub body_json: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub body_text: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ContactListResult {
    pub profile: String,
    pub context: ResolvedContext,
    pub location_id: String,
    pub limit: u32,
    pub endpoint: String,
    pub url: String,
    pub status: u16,
    pub success: bool,
    pub count: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub total: Option<u64>,
    pub contact_ids: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ContactSearchDryRun {
    pub method: &'static str,
    pub surface: &'static str,
    pub path: String,
    pub context: ResolvedContext,
    pub location_id: String,
    pub request_body_json: Value,
    pub auth_class: &'static str,
    pub network: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ContactListDryRun {
    pub method: &'static str,
    pub surface: &'static str,
    pub path: String,
    pub context: ResolvedContext,
    pub location_id: String,
    pub request_body_json: Value,
    pub auth_class: &'static str,
    pub network: bool,
}

pub fn get_contact(
    paths: &crate::ConfigPaths,
    profile_name: Option<&str>,
    location_override: Option<&str>,
    contact_id: &str,
) -> Result<ContactGetResult> {
    validate_contact_id(contact_id)?;
    let context = resolve_context(paths, profile_name, location_override, None)?;
    let location_id = context.require_location_id()?.to_owned();
    let endpoint = contact_endpoint(contact_id);
    let response = raw_get(
        paths,
        profile_name,
        RawGetRequest {
            surface: Surface::Services,
            path: endpoint.clone(),
            auth_class: AuthClass::Pit,
            include_body: true,
        },
    )?;

    Ok(ContactGetResult {
        profile: context.profile.clone(),
        context,
        location_id,
        contact_id: contact_id.to_owned(),
        endpoint,
        url: response.url,
        status: response.status,
        success: response.success,
        body_json: response.body_json.as_ref().map(redact_contact_write_body),
        body_text: response.body_text,
    })
}

pub fn list_contacts(
    paths: &crate::ConfigPaths,
    profile_name: Option<&str>,
    location_override: Option<&str>,
    options: ContactListOptions,
) -> Result<ContactListResult> {
    validate_list_options(&options)?;
    let context = resolve_context(paths, profile_name, location_override, None)?;
    let location_id = context.require_location_id()?.to_owned();
    let endpoint = contacts_search_endpoint();
    let request_body = contacts_list_body(&location_id, &options);
    let response = post_json(
        paths,
        profile_name,
        RawPostJsonRequest {
            surface: Surface::Services,
            path: endpoint.to_owned(),
            auth_class: AuthClass::Pit,
            body: request_body,
            include_body: true,
        },
    )?;
    let (count, total, contact_ids) = summarize_contact_page(response.body_json.as_ref());

    Ok(ContactListResult {
        profile: context.profile.clone(),
        context,
        location_id,
        limit: options.limit,
        endpoint: endpoint.to_owned(),
        url: response.url,
        status: response.status,
        success: response.success,
        count,
        total,
        contact_ids,
    })
}

pub fn search_contacts(
    paths: &crate::ConfigPaths,
    profile_name: Option<&str>,
    location_override: Option<&str>,
    options: ContactSearchOptions,
) -> Result<ContactSearchResult> {
    validate_search_options(&options)?;
    let context = resolve_context(paths, profile_name, location_override, None)?;
    let location_id = context.require_location_id()?.to_owned();
    let request_body = contacts_search_body(&location_id, &options);
    let endpoint = contacts_search_endpoint();
    let response = post_json(
        paths,
        profile_name,
        RawPostJsonRequest {
            surface: Surface::Services,
            path: endpoint.to_owned(),
            auth_class: AuthClass::Pit,
            body: request_body,
            include_body: true,
        },
    )?;

    Ok(ContactSearchResult {
        profile: context.profile.clone(),
        context,
        location_id,
        query: trimmed_optional(options.query),
        email_filter: trimmed_optional(options.email),
        phone_filter: trimmed_optional(options.phone),
        limit: options.limit,
        endpoint: endpoint.to_owned(),
        url: response.url,
        status: response.status,
        success: response.success,
        body_json: response.body_json.as_ref().map(redact_contact_write_body),
        body_text: response.body_text,
    })
}

pub fn contacts_list_dry_run(
    paths: &crate::ConfigPaths,
    profile_name: Option<&str>,
    location_override: Option<&str>,
    options: ContactListOptions,
) -> Result<ContactListDryRun> {
    validate_list_options(&options)?;
    let context = resolve_context_for_dry_run(paths, profile_name, location_override, None)?;
    let location_id = context.require_location_id()?.to_owned();
    let request_body_json = contacts_list_body(&location_id, &options);

    Ok(ContactListDryRun {
        method: "POST",
        surface: "services",
        path: contacts_search_endpoint().to_owned(),
        context,
        location_id,
        request_body_json,
        auth_class: "pit",
        network: false,
    })
}

pub fn get_contact_dry_run(
    paths: &crate::ConfigPaths,
    profile_name: Option<&str>,
    location_override: Option<&str>,
    contact_id: &str,
) -> Result<ContactGetDryRun> {
    validate_contact_id(contact_id)?;
    let context = resolve_context_for_dry_run(paths, profile_name, location_override, None)?;
    let location_id = context.require_location_id()?.to_owned();

    Ok(ContactGetDryRun {
        method: "GET",
        surface: "services",
        path: contact_endpoint(contact_id),
        context,
        location_id,
        contact_id: contact_id.to_owned(),
        auth_class: "pit",
        network: false,
    })
}

pub fn contacts_search_dry_run(
    paths: &crate::ConfigPaths,
    profile_name: Option<&str>,
    location_override: Option<&str>,
    options: ContactSearchOptions,
) -> Result<ContactSearchDryRun> {
    validate_search_options(&options)?;
    let context = resolve_context_for_dry_run(paths, profile_name, location_override, None)?;
    let location_id = context.require_location_id()?.to_owned();
    let request_body_json = contacts_search_body(&location_id, &options);

    Ok(ContactSearchDryRun {
        method: "POST",
        surface: "services",
        path: contacts_search_endpoint().to_owned(),
        context,
        location_id,
        request_body_json,
        auth_class: "pit",
        network: false,
    })
}

pub fn create_contact_dry_run(
    paths: &crate::ConfigPaths,
    profile_name: Option<&str>,
    location_override: Option<&str>,
    options: ContactCreateOptions,
) -> Result<ContactWriteDryRun> {
    validate_create_options(&options)?;
    let context = resolve_context_for_dry_run(paths, profile_name, location_override, None)?;
    let location_id = context.require_location_id()?.to_owned();
    let endpoint = contact_create_endpoint().to_owned();
    let request_body_json = contact_create_body(&location_id, &options.fields);
    let request_hash = stable_request_hash(&json!({
        "method": "POST",
        "path": endpoint,
        "body": request_body_json,
    }))?;
    let preflight = dry_run_duplicate_preflight(&options.fields);
    let audit = write_contact_dry_run_audit(
        paths,
        &context,
        "contacts.create",
        "contact create dry-run; no network mutation performed",
        &endpoint,
        None,
        &request_body_json,
        &request_hash,
        &preflight,
    )?;

    Ok(ContactWriteDryRun {
        method: "POST",
        surface: "services",
        path: endpoint,
        context,
        location_id,
        contact_id: None,
        request_body_json: redact_contact_write_body(&request_body_json),
        request_hash,
        idempotency_key: options.idempotency_key,
        preflight,
        audit_entry_id: audit.id,
        auth_class: "pit",
        network: false,
    })
}

pub fn update_contact_dry_run(
    paths: &crate::ConfigPaths,
    profile_name: Option<&str>,
    location_override: Option<&str>,
    options: ContactUpdateOptions,
) -> Result<ContactWriteDryRun> {
    validate_update_options(&options)?;
    let context = resolve_context_for_dry_run(paths, profile_name, location_override, None)?;
    let location_id = context.require_location_id()?.to_owned();
    let endpoint = contact_endpoint(&options.contact_id);
    let request_body_json = contact_update_body(&options.fields);
    let request_hash = stable_request_hash(&json!({
        "method": "PUT",
        "path": endpoint,
        "body": request_body_json,
    }))?;
    let preflight = skipped_duplicate_preflight();
    let audit = write_contact_dry_run_audit(
        paths,
        &context,
        "contacts.update",
        "contact update dry-run; no network mutation performed",
        &endpoint,
        Some(options.contact_id.clone()),
        &request_body_json,
        &request_hash,
        &preflight,
    )?;

    Ok(ContactWriteDryRun {
        method: "PUT",
        surface: "services",
        path: endpoint,
        context,
        location_id,
        contact_id: Some(options.contact_id),
        request_body_json: redact_contact_write_body(&request_body_json),
        request_hash,
        idempotency_key: options.idempotency_key,
        preflight,
        audit_entry_id: audit.id,
        auth_class: "pit",
        network: false,
    })
}

pub fn create_contact(
    paths: &crate::ConfigPaths,
    profile_name: Option<&str>,
    location_override: Option<&str>,
    options: ContactCreateOptions,
) -> Result<ContactWriteResult> {
    validate_create_options(&options)?;
    let context = resolve_context(paths, profile_name, location_override, None)?;
    ensure_write_allowed(paths, &context.profile)?;
    let location_id = context.require_location_id()?.to_owned();
    let endpoint = contact_create_endpoint().to_owned();
    let request_body = contact_create_body(&location_id, &options.fields);
    let request_hash = stable_request_hash(&json!({
        "method": "POST",
        "path": endpoint,
        "body": request_body,
    }))?;
    let idempotency_key =
        required_idempotency_key(&options.idempotency_key, "real contact create")?;

    if let Some(replay) = maybe_replay_contact_write(
        paths,
        &context,
        "contacts.create",
        &endpoint,
        &idempotency_key,
        &request_hash,
        None,
        skipped_duplicate_preflight(),
    )? {
        return Ok(replay);
    }

    let preflight = real_duplicate_preflight(paths, profile_name, &location_id, &options.fields)?;
    record_contact_in_progress(
        paths,
        &context,
        "contacts.create",
        &idempotency_key,
        &request_hash,
        None,
    )?;

    let response = post_json(
        paths,
        profile_name,
        RawPostJsonRequest {
            surface: Surface::Services,
            path: endpoint.clone(),
            auth_class: AuthClass::Pit,
            body: request_body.clone(),
            include_body: true,
        },
    )?;
    let contact_id = extract_contact_id(response.body_json.as_ref());
    let audit = write_contact_result_audit(
        paths,
        &context,
        "contacts.create",
        &endpoint,
        None,
        &request_body,
        &request_hash,
        &idempotency_key,
        contact_id.clone(),
        &preflight,
        response.status,
        response.headers.get("x-request-id").cloned(),
        response.success,
        response
            .body_text
            .clone()
            .or_else(|| response.body_json.as_ref().map(Value::to_string)),
    )?;
    record_contact_done(
        paths,
        &context,
        ContactDone {
            command: "contacts.create",
            key: &idempotency_key,
            request_hash: &request_hash,
            contact_id: contact_id.clone(),
            audit_entry_id: Some(audit.id.clone()),
            success: response.success,
        },
    )?;

    Ok(ContactWriteResult {
        profile: context.profile.clone(),
        context,
        location_id,
        endpoint,
        url: response.url,
        status: response.status,
        success: response.success,
        replayed: false,
        contact_id,
        idempotency_key: Some(idempotency_key),
        idempotency_state: Some("recorded".to_owned()),
        request_hash,
        preflight,
        audit_entry_id: audit.id,
        body_json: response.body_json.as_ref().map(redact_contact_write_body),
        body_text: response.body_text,
    })
}

pub fn update_contact(
    paths: &crate::ConfigPaths,
    profile_name: Option<&str>,
    location_override: Option<&str>,
    options: ContactUpdateOptions,
) -> Result<ContactWriteResult> {
    validate_update_options(&options)?;
    let context = resolve_context(paths, profile_name, location_override, None)?;
    ensure_write_allowed(paths, &context.profile)?;
    let location_id = context.require_location_id()?.to_owned();
    let endpoint = contact_endpoint(&options.contact_id);
    let request_body = contact_update_body(&options.fields);
    let request_hash = stable_request_hash(&json!({
        "method": "PUT",
        "path": endpoint,
        "body": request_body,
    }))?;
    let idempotency_key =
        required_idempotency_key(&options.idempotency_key, "real contact update")?;
    let preflight = skipped_duplicate_preflight();

    if let Some(replay) = maybe_replay_contact_write(
        paths,
        &context,
        "contacts.update",
        &endpoint,
        &idempotency_key,
        &request_hash,
        Some(options.contact_id.clone()),
        preflight.clone(),
    )? {
        return Ok(replay);
    }

    record_contact_in_progress(
        paths,
        &context,
        "contacts.update",
        &idempotency_key,
        &request_hash,
        Some(options.contact_id.clone()),
    )?;
    let response = put_json(
        paths,
        profile_name,
        RawPutJsonRequest {
            surface: Surface::Services,
            path: endpoint.clone(),
            auth_class: AuthClass::Pit,
            body: request_body.clone(),
            include_body: true,
        },
    )?;
    let contact_id = extract_contact_id(response.body_json.as_ref()).or(Some(options.contact_id));
    let audit = write_contact_result_audit(
        paths,
        &context,
        "contacts.update",
        &endpoint,
        contact_id.clone(),
        &request_body,
        &request_hash,
        &idempotency_key,
        contact_id.clone(),
        &preflight,
        response.status,
        response.headers.get("x-request-id").cloned(),
        response.success,
        response
            .body_text
            .clone()
            .or_else(|| response.body_json.as_ref().map(Value::to_string)),
    )?;
    record_contact_done(
        paths,
        &context,
        ContactDone {
            command: "contacts.update",
            key: &idempotency_key,
            request_hash: &request_hash,
            contact_id: contact_id.clone(),
            audit_entry_id: Some(audit.id.clone()),
            success: response.success,
        },
    )?;

    Ok(ContactWriteResult {
        profile: context.profile.clone(),
        context,
        location_id,
        endpoint,
        url: response.url,
        status: response.status,
        success: response.success,
        replayed: false,
        contact_id,
        idempotency_key: Some(idempotency_key),
        idempotency_state: Some("recorded".to_owned()),
        request_hash,
        preflight,
        audit_entry_id: audit.id,
        body_json: response.body_json.as_ref().map(redact_contact_write_body),
        body_text: response.body_text,
    })
}

fn redact_contact_write_body(value: &Value) -> Value {
    match value {
        Value::Object(map) => Value::Object(
            map.iter()
                .map(|(key, value)| {
                    if key.eq_ignore_ascii_case("email") || key.eq_ignore_ascii_case("phone") {
                        (key.clone(), Value::String("[REDACTED]".to_owned()))
                    } else {
                        (key.clone(), redact_contact_write_body(value))
                    }
                })
                .collect(),
        ),
        Value::Array(items) => Value::Array(items.iter().map(redact_contact_write_body).collect()),
        other => redact_json(other),
    }
}

fn contact_create_endpoint() -> &'static str {
    "/contacts/"
}

fn contact_create_body(location_id: &str, fields: &ContactWriteFields) -> Value {
    let mut body = contact_update_map(fields);
    body.insert(
        "locationId".to_owned(),
        Value::String(location_id.to_owned()),
    );
    Value::Object(body)
}

fn contact_update_body(fields: &ContactWriteFields) -> Value {
    Value::Object(contact_update_map(fields))
}

fn contact_update_map(fields: &ContactWriteFields) -> Map<String, Value> {
    let mut body = Map::new();
    insert_optional_string(&mut body, "firstName", fields.first_name.as_deref());
    insert_optional_string(&mut body, "lastName", fields.last_name.as_deref());
    insert_optional_string(&mut body, "name", fields.name.as_deref());
    insert_optional_string(&mut body, "email", fields.email.as_deref());
    insert_optional_string(&mut body, "phone", fields.phone.as_deref());
    insert_optional_string(&mut body, "address1", fields.address1.as_deref());
    insert_optional_string(&mut body, "city", fields.city.as_deref());
    insert_optional_string(&mut body, "state", fields.state.as_deref());
    insert_optional_string(&mut body, "country", fields.country.as_deref());
    insert_optional_string(&mut body, "postalCode", fields.postal_code.as_deref());
    insert_optional_string(&mut body, "website", fields.website.as_deref());
    insert_optional_string(&mut body, "timezone", fields.timezone.as_deref());
    insert_optional_string(&mut body, "companyName", fields.company_name.as_deref());
    insert_optional_string(&mut body, "source", fields.source.as_deref());
    insert_optional_string(&mut body, "assignedTo", fields.assigned_to.as_deref());
    let tags = fields
        .tags
        .iter()
        .filter_map(|tag| trimmed_optional(Some(tag.clone())))
        .map(Value::String)
        .collect::<Vec<_>>();
    if !tags.is_empty() {
        body.insert("tags".to_owned(), Value::Array(tags));
    }
    body
}

fn insert_optional_string(body: &mut Map<String, Value>, key: &str, value: Option<&str>) {
    if let Some(value) = value.and_then(|value| trimmed_optional(Some(value.to_owned()))) {
        body.insert(key.to_owned(), Value::String(value));
    }
}

fn write_policy_flags() -> Vec<String> {
    vec![
        "allow_destructive".to_owned(),
        "confirmation_required".to_owned(),
        "idempotency_key_required".to_owned(),
    ]
}

#[allow(clippy::too_many_arguments)]
fn write_contact_dry_run_audit(
    paths: &crate::ConfigPaths,
    context: &ResolvedContext,
    command: &str,
    message: &str,
    endpoint: &str,
    contact_id: Option<String>,
    request_body_json: &Value,
    request_hash: &str,
    preflight: &ContactDuplicatePreflight,
) -> Result<crate::audit::AuditEntry> {
    append_audit_entry(
        paths,
        AuditEntryInput {
            profile: Some(context.profile.clone()),
            company_id: context.company_id.as_ref().map(|value| value.value.clone()),
            location_id: context
                .location_id
                .as_ref()
                .map(|value| value.value.clone()),
            command: command.to_owned(),
            action_class: "sensitive_dry_run".to_owned(),
            dry_run: true,
            policy_flags: write_policy_flags(),
            resource: Some(AuditResource {
                resource_type: "contact".to_owned(),
                id: contact_id.clone(),
            }),
            request_summary: json!({
                "endpoint": endpoint,
                "request_body": redact_contact_write_body(request_body_json),
                "request_hash": request_hash,
                "preflight": preflight,
            }),
            upstream: None,
            result: AuditResultSummary {
                status: "dry_run".to_owned(),
                resource_id: contact_id,
                message: Some(message.to_owned()),
            },
            error: None,
        },
    )
}

#[allow(clippy::too_many_arguments)]
fn write_contact_result_audit(
    paths: &crate::ConfigPaths,
    context: &ResolvedContext,
    command: &str,
    endpoint: &str,
    resource_contact_id: Option<String>,
    request_body: &Value,
    request_hash: &str,
    idempotency_key: &str,
    result_contact_id: Option<String>,
    preflight: &ContactDuplicatePreflight,
    status: u16,
    request_id: Option<String>,
    success: bool,
    error_body: Option<String>,
) -> Result<crate::audit::AuditEntry> {
    append_audit_entry(
        paths,
        AuditEntryInput {
            profile: Some(context.profile.clone()),
            company_id: context.company_id.as_ref().map(|value| value.value.clone()),
            location_id: context
                .location_id
                .as_ref()
                .map(|value| value.value.clone()),
            command: command.to_owned(),
            action_class: "write".to_owned(),
            dry_run: false,
            policy_flags: write_policy_flags(),
            resource: Some(AuditResource {
                resource_type: "contact".to_owned(),
                id: resource_contact_id.or_else(|| result_contact_id.clone()),
            }),
            request_summary: json!({
                "endpoint": endpoint,
                "request_body": redact_contact_write_body(request_body),
                "request_hash": request_hash,
                "idempotency_key": idempotency_key,
                "preflight": preflight,
            }),
            upstream: Some(AuditUpstreamSummary {
                request_id,
                status_code: Some(status),
                endpoint_key: Some(command.to_owned()),
            }),
            result: AuditResultSummary {
                status: if success { "success" } else { "failed" }.to_owned(),
                resource_id: result_contact_id,
                message: None,
            },
            error: if success { None } else { error_body },
        },
    )
}

#[allow(clippy::too_many_arguments)]
fn maybe_replay_contact_write(
    paths: &crate::ConfigPaths,
    context: &ResolvedContext,
    command: &str,
    endpoint: &str,
    idempotency_key: &str,
    request_hash: &str,
    fallback_contact_id: Option<String>,
    preflight: ContactDuplicatePreflight,
) -> Result<Option<ContactWriteResult>> {
    let location_id = context.require_location_id()?.to_owned();
    let idempotency = check_idempotency_key(
        paths,
        &context.profile,
        Some(&location_id),
        command,
        idempotency_key,
        request_hash,
    )?;
    if idempotency.state != IdempotencyCheckState::Replay {
        return Ok(None);
    }
    let existing = idempotency.existing.expect("existing replay record");
    let contact_id = existing.resource_id.clone().or(fallback_contact_id);
    let audit = append_audit_entry(
        paths,
        AuditEntryInput {
            profile: Some(context.profile.clone()),
            company_id: context.company_id.as_ref().map(|value| value.value.clone()),
            location_id: Some(location_id.clone()),
            command: command.to_owned(),
            action_class: "idempotency_replay".to_owned(),
            dry_run: false,
            policy_flags: write_policy_flags(),
            resource: Some(AuditResource {
                resource_type: "contact".to_owned(),
                id: contact_id.clone(),
            }),
            request_summary: json!({
                "endpoint": endpoint,
                "idempotency_key": idempotency_key,
                "request_hash": request_hash,
            }),
            upstream: None,
            result: AuditResultSummary {
                status: "replayed".to_owned(),
                resource_id: contact_id.clone(),
                message: Some(
                    "idempotency key matched a previous request; no network mutation performed"
                        .to_owned(),
                ),
            },
            error: None,
        },
    )?;

    Ok(Some(ContactWriteResult {
        profile: context.profile.clone(),
        context: context.clone(),
        location_id,
        endpoint: endpoint.to_owned(),
        url: String::new(),
        status: 200,
        success: existing.status == IdempotencyStatus::Succeeded,
        replayed: true,
        contact_id,
        idempotency_key: Some(idempotency_key.to_owned()),
        idempotency_state: Some("replay".to_owned()),
        request_hash: request_hash.to_owned(),
        preflight,
        audit_entry_id: audit.id,
        body_json: None,
        body_text: None,
    }))
}

fn record_contact_in_progress(
    paths: &crate::ConfigPaths,
    context: &ResolvedContext,
    command: &str,
    key: &str,
    request_hash: &str,
    contact_id: Option<String>,
) -> Result<()> {
    record_idempotency_key(
        paths,
        IdempotencyPut {
            key: key.to_owned(),
            profile: context.profile.clone(),
            location_id: context
                .location_id
                .as_ref()
                .map(|value| value.value.clone()),
            command: command.to_owned(),
            request_hash: request_hash.to_owned(),
            status: IdempotencyStatus::InProgress,
            resource_id: contact_id,
            audit_entry_id: None,
        },
    )?;
    Ok(())
}

struct ContactDone<'a> {
    command: &'a str,
    key: &'a str,
    request_hash: &'a str,
    contact_id: Option<String>,
    audit_entry_id: Option<String>,
    success: bool,
}

fn record_contact_done(
    paths: &crate::ConfigPaths,
    context: &ResolvedContext,
    done: ContactDone<'_>,
) -> Result<()> {
    record_idempotency_key(
        paths,
        IdempotencyPut {
            key: done.key.to_owned(),
            profile: context.profile.clone(),
            location_id: context
                .location_id
                .as_ref()
                .map(|value| value.value.clone()),
            command: done.command.to_owned(),
            request_hash: done.request_hash.to_owned(),
            status: if done.success {
                IdempotencyStatus::Succeeded
            } else {
                IdempotencyStatus::Failed
            },
            resource_id: done.contact_id,
            audit_entry_id: done.audit_entry_id,
        },
    )?;
    Ok(())
}

fn dry_run_duplicate_preflight(fields: &ContactWriteFields) -> ContactDuplicatePreflight {
    let email_checked = has_value(fields.email.as_deref());
    let phone_checked = has_value(fields.phone.as_deref());
    ContactDuplicatePreflight {
        status: if email_checked || phone_checked {
            "planned"
        } else {
            "skipped"
        }
        .to_owned(),
        checked: false,
        email_checked,
        phone_checked,
        duplicate_count: 0,
        duplicate_contact_ids: Vec::new(),
    }
}

fn skipped_duplicate_preflight() -> ContactDuplicatePreflight {
    ContactDuplicatePreflight {
        status: "skipped".to_owned(),
        checked: false,
        email_checked: false,
        phone_checked: false,
        duplicate_count: 0,
        duplicate_contact_ids: Vec::new(),
    }
}

fn real_duplicate_preflight(
    paths: &crate::ConfigPaths,
    profile_name: Option<&str>,
    location_id: &str,
    fields: &ContactWriteFields,
) -> Result<ContactDuplicatePreflight> {
    let email = trimmed_optional(fields.email.clone());
    let phone = trimmed_optional(fields.phone.clone());
    if email.is_none() && phone.is_none() {
        return Ok(skipped_duplicate_preflight());
    }

    let mut ids = Vec::new();
    if let Some(email) = email {
        ids.extend(duplicate_ids_for_exact_filter(
            paths,
            profile_name,
            location_id,
            Some(email),
            None,
        )?);
    }
    if let Some(phone) = phone {
        ids.extend(duplicate_ids_for_exact_filter(
            paths,
            profile_name,
            location_id,
            None,
            Some(phone),
        )?);
    }
    ids.sort();
    ids.dedup();
    if !ids.is_empty() {
        return Err(GhlError::Validation {
            message: format!(
                "contact create duplicate preflight found existing contact ids: {}",
                ids.join(",")
            ),
        });
    }

    Ok(ContactDuplicatePreflight {
        status: "passed".to_owned(),
        checked: true,
        email_checked: has_value(fields.email.as_deref()),
        phone_checked: has_value(fields.phone.as_deref()),
        duplicate_count: 0,
        duplicate_contact_ids: Vec::new(),
    })
}

fn duplicate_ids_for_exact_filter(
    paths: &crate::ConfigPaths,
    profile_name: Option<&str>,
    location_id: &str,
    email: Option<String>,
    phone: Option<String>,
) -> Result<Vec<String>> {
    let response = post_json(
        paths,
        profile_name,
        RawPostJsonRequest {
            surface: Surface::Services,
            path: contacts_search_endpoint().to_owned(),
            auth_class: AuthClass::Pit,
            body: contacts_search_body_parts(location_id, 5, None, email, phone, None, None),
            include_body: true,
        },
    )?;
    let (_, _, ids) = summarize_contact_page(response.body_json.as_ref());
    Ok(ids)
}

fn extract_contact_id(body: Option<&Value>) -> Option<String> {
    let body = body?;
    body.get("contact")
        .and_then(|contact| {
            contact
                .get("id")
                .or_else(|| contact.get("contactId"))
                .and_then(Value::as_str)
        })
        .or_else(|| body.get("id").and_then(Value::as_str))
        .or_else(|| body.get("contactId").and_then(Value::as_str))
        .map(str::to_owned)
}

fn required_idempotency_key(value: &Option<String>, command: &str) -> Result<String> {
    trimmed_optional(value.clone()).ok_or_else(|| GhlError::Validation {
        message: format!("{command} requires --idempotency-key <key>"),
    })
}

fn ensure_write_allowed(paths: &crate::ConfigPaths, profile_name: &str) -> Result<()> {
    let profiles = load_profiles(paths)?;
    let profile = profiles.get_required(profile_name)?;
    if !profile.policy.allow_destructive {
        return Err(GhlError::PolicyDenied {
            message: "profile policy blocks contact writes; enable allow_destructive for this profile before real contact mutations".to_owned(),
        });
    }
    Ok(())
}

fn validate_create_options(options: &ContactCreateOptions) -> Result<()> {
    validate_write_fields(&options.fields, true)?;
    validate_optional_text(
        options.idempotency_key.as_deref(),
        "contact idempotency key",
    )
}

fn validate_update_options(options: &ContactUpdateOptions) -> Result<()> {
    validate_contact_id(&options.contact_id)?;
    validate_write_fields(&options.fields, false)?;
    validate_optional_text(
        options.idempotency_key.as_deref(),
        "contact idempotency key",
    )
}

fn validate_write_fields(fields: &ContactWriteFields, require_identity: bool) -> Result<()> {
    for (label, value) in [
        ("contact first name", fields.first_name.as_deref()),
        ("contact last name", fields.last_name.as_deref()),
        ("contact name", fields.name.as_deref()),
        ("contact email", fields.email.as_deref()),
        ("contact phone", fields.phone.as_deref()),
        ("contact address", fields.address1.as_deref()),
        ("contact city", fields.city.as_deref()),
        ("contact state", fields.state.as_deref()),
        ("contact country", fields.country.as_deref()),
        ("contact postal code", fields.postal_code.as_deref()),
        ("contact website", fields.website.as_deref()),
        ("contact timezone", fields.timezone.as_deref()),
        ("contact company name", fields.company_name.as_deref()),
        ("contact source", fields.source.as_deref()),
        ("contact assigned user", fields.assigned_to.as_deref()),
    ] {
        validate_optional_text(value, label)?;
    }
    for tag in &fields.tags {
        validate_optional_text(Some(tag), "contact tag")?;
    }
    let has_any_field = !contact_update_map(fields).is_empty();
    if !has_any_field {
        return Err(GhlError::Validation {
            message: "contact write requires at least one field".to_owned(),
        });
    }
    if require_identity
        && !has_value(fields.name.as_deref())
        && !has_value(fields.first_name.as_deref())
        && !has_value(fields.last_name.as_deref())
        && !has_value(fields.email.as_deref())
        && !has_value(fields.phone.as_deref())
    {
        return Err(GhlError::Validation {
            message: "contact create requires a name, first/last name, email, or phone".to_owned(),
        });
    }
    Ok(())
}

fn contact_endpoint(contact_id: &str) -> String {
    format!("/contacts/{contact_id}")
}

fn contacts_search_endpoint() -> &'static str {
    "/contacts/search"
}

fn contacts_list_body(location_id: &str, options: &ContactListOptions) -> Value {
    contacts_search_body_parts(
        location_id,
        options.limit,
        None,
        None,
        None,
        options.start_after_id.clone(),
        options.start_after,
    )
}

fn contacts_search_body(location_id: &str, options: &ContactSearchOptions) -> Value {
    contacts_search_body_parts(
        location_id,
        options.limit,
        options.query.clone(),
        options.email.clone(),
        options.phone.clone(),
        options.start_after_id.clone(),
        options.start_after,
    )
}

fn contacts_search_body_parts(
    location_id: &str,
    limit: u32,
    query: Option<String>,
    email: Option<String>,
    phone: Option<String>,
    start_after_id: Option<String>,
    start_after: Option<u64>,
) -> Value {
    let mut body = Map::new();
    body.insert(
        "locationId".to_owned(),
        Value::String(location_id.to_owned()),
    );
    body.insert(
        "pageLimit".to_owned(),
        Value::Number(serde_json::Number::from(limit)),
    );
    if let Some(query) = trimmed_optional(query) {
        body.insert("query".to_owned(), Value::String(query));
    }
    if let Some(start_after_id) = trimmed_optional(start_after_id) {
        body.insert("startAfterId".to_owned(), Value::String(start_after_id));
    }
    if let Some(start_after) = start_after {
        body.insert(
            "startAfter".to_owned(),
            Value::Number(serde_json::Number::from(start_after)),
        );
    }

    let mut filters = Vec::new();
    if let Some(email) = trimmed_optional(email) {
        filters.push(exact_filter("email", email));
    }
    if let Some(phone) = trimmed_optional(phone) {
        filters.push(exact_filter("phone", phone));
    }
    if !filters.is_empty() {
        body.insert("filters".to_owned(), Value::Array(filters));
    }

    Value::Object(body)
}

fn exact_filter(field: &str, value: String) -> Value {
    let mut filter = Map::new();
    filter.insert("field".to_owned(), Value::String(field.to_owned()));
    filter.insert("operator".to_owned(), Value::String("eq".to_owned()));
    filter.insert("value".to_owned(), Value::String(value));
    Value::Object(filter)
}

fn summarize_contact_page(body: Option<&Value>) -> (usize, Option<u64>, Vec<String>) {
    let Some(body) = body else {
        return (0, None, Vec::new());
    };
    let contacts = body
        .get("contacts")
        .and_then(Value::as_array)
        .map(Vec::as_slice)
        .unwrap_or(&[]);
    let contact_ids = contacts
        .iter()
        .filter_map(|contact| {
            contact
                .get("id")
                .or_else(|| contact.get("contactId"))
                .and_then(Value::as_str)
                .map(str::to_owned)
        })
        .collect::<Vec<_>>();
    let total = body.get("total").and_then(Value::as_u64);

    (contacts.len(), total, contact_ids)
}

fn validate_contact_id(contact_id: &str) -> Result<()> {
    validate_path_segment(contact_id, "contact id")
}

fn validate_list_options(options: &ContactListOptions) -> Result<()> {
    validate_limit(options.limit, "contact list")?;
    validate_optional_text(options.start_after_id.as_deref(), "contact start-after id")?;

    Ok(())
}

fn validate_search_options(options: &ContactSearchOptions) -> Result<()> {
    let has_query = has_value(options.query.as_deref());
    let has_email = has_value(options.email.as_deref());
    let has_phone = has_value(options.phone.as_deref());
    if !has_query && !has_email && !has_phone {
        return Err(GhlError::Validation {
            message: "contact search needs a query, --email, or --phone".to_owned(),
        });
    }
    validate_limit(options.limit, "contact search")?;
    validate_optional_text(options.query.as_deref(), "contact search query")?;
    validate_optional_text(options.email.as_deref(), "contact email filter")?;
    validate_optional_text(options.phone.as_deref(), "contact phone filter")?;
    validate_optional_text(options.start_after_id.as_deref(), "contact start-after id")?;

    Ok(())
}

fn validate_limit(limit: u32, label: &str) -> Result<()> {
    if limit == 0 || limit > 100 {
        return Err(GhlError::Validation {
            message: format!("{label} limit must be between 1 and 100"),
        });
    }

    Ok(())
}

fn validate_optional_text(value: Option<&str>, label: &str) -> Result<()> {
    if let Some(value) = value {
        if value.trim().is_empty() {
            return Err(GhlError::Validation {
                message: format!("{label} cannot be empty"),
            });
        }
        if value.chars().any(|character| character.is_ascii_control()) {
            return Err(GhlError::Validation {
                message: format!("{label} cannot contain control characters"),
            });
        }
    }

    Ok(())
}

fn validate_path_segment(value: &str, label: &str) -> Result<()> {
    if value.trim().is_empty() {
        return Err(GhlError::Validation {
            message: format!("{label} cannot be empty"),
        });
    }
    if value.chars().any(|character| {
        character == '/'
            || character == '?'
            || character == '#'
            || character.is_ascii_control()
            || character.is_whitespace()
    }) {
        return Err(GhlError::Validation {
            message: format!("{label} must be a single path segment"),
        });
    }

    Ok(())
}

fn has_value(value: Option<&str>) -> bool {
    value.is_some_and(|value| !value.trim().is_empty())
}

fn trimmed_optional(value: Option<String>) -> Option<String> {
    value
        .map(|value| value.trim().to_owned())
        .filter(|value| !value.is_empty())
}

#[cfg(test)]
mod tests {
    use httpmock::Method::{GET, POST, PUT};
    use httpmock::MockServer;
    use serde_json::json;

    use super::*;
    use crate::auth::add_pit;
    use crate::config::resolve_paths;

    #[test]
    fn contact_get_requires_location_context_and_redacts_body() {
        let server = MockServer::start();
        let mock = server.mock(|when, then| {
            when.method(GET)
                .path("/contacts/contact_123")
                .header("authorization", "Bearer pit-secret");
            then.status(200)
                .header("content-type", "application/json")
                .json_body(json!({
                    "contact": {
                        "id": "contact_123",
                        "name": "John Doe",
                        "apiKey": "secret-value"
                    }
                }));
        });
        let temp = tempfile::tempdir().expect("tempdir");
        let paths = resolve_paths(Some(temp.path()));
        add_pit(
            &paths,
            "default",
            "pit-secret".to_owned(),
            Some("loc_123".to_owned()),
            None,
            true,
        )
        .expect("pit");
        let mut profiles = crate::profiles::load_profiles(&paths).expect("profiles");
        profiles
            .profiles
            .get_mut("default")
            .expect("profile")
            .base_urls
            .services = server.base_url();
        crate::profiles::save_profiles(&paths, &profiles).expect("save");

        let result = get_contact(&paths, Some("default"), None, "contact_123").expect("contact");

        mock.assert();
        assert_eq!(result.location_id, "loc_123");
        assert_eq!(result.status, 200);
        assert_eq!(result.body_json.unwrap()["contact"]["apiKey"], "[REDACTED]");
    }

    #[test]
    fn contact_list_returns_summary_without_contact_bodies() {
        let server = MockServer::start();
        let mock = server.mock(|when, then| {
            when.method(POST)
                .path("/contacts/search")
                .header("authorization", "Bearer pit-secret")
                .json_body(json!({
                    "locationId": "loc_123",
                    "pageLimit": 5
                }));
            then.status(200)
                .header("content-type", "application/json")
                .json_body(json!({
                    "contacts": [
                        { "id": "contact_123", "email": "john@example.com", "phone": "+15551234567" }
                    ],
                    "total": 1
                }));
        });
        let temp = tempfile::tempdir().expect("tempdir");
        let paths = resolve_paths(Some(temp.path()));
        add_pit(
            &paths,
            "default",
            "pit-secret".to_owned(),
            Some("loc_123".to_owned()),
            None,
            true,
        )
        .expect("pit");
        let mut profiles = crate::profiles::load_profiles(&paths).expect("profiles");
        profiles
            .profiles
            .get_mut("default")
            .expect("profile")
            .base_urls
            .services = server.base_url();
        crate::profiles::save_profiles(&paths, &profiles).expect("save");

        let result = list_contacts(
            &paths,
            Some("default"),
            None,
            ContactListOptions {
                limit: 5,
                start_after_id: None,
                start_after: None,
            },
        )
        .expect("contacts");

        mock.assert();
        assert_eq!(result.location_id, "loc_123");
        assert_eq!(result.status, 200);
        assert_eq!(result.count, 1);
        assert_eq!(result.total, Some(1));
        assert_eq!(result.contact_ids, vec!["contact_123".to_owned()]);
    }

    #[test]
    fn contact_search_posts_location_query_and_exact_filters() {
        let server = MockServer::start();
        let mock = server.mock(|when, then| {
            when.method(POST)
                .path("/contacts/search")
                .header("authorization", "Bearer pit-secret")
                .json_body(json!({
                    "locationId": "loc_123",
                    "pageLimit": 25,
                    "query": "John",
                    "filters": [
                        { "field": "email", "operator": "eq", "value": "john@example.com" }
                    ]
                }));
            then.status(200)
                .header("content-type", "application/json")
                .json_body(json!({
                    "contacts": [
                        { "id": "contact_123", "email": "john@example.com", "token": "secret-token" }
                    ]
                }));
        });
        let temp = tempfile::tempdir().expect("tempdir");
        let paths = resolve_paths(Some(temp.path()));
        add_pit(
            &paths,
            "default",
            "pit-secret".to_owned(),
            Some("loc_123".to_owned()),
            None,
            true,
        )
        .expect("pit");
        let mut profiles = crate::profiles::load_profiles(&paths).expect("profiles");
        profiles
            .profiles
            .get_mut("default")
            .expect("profile")
            .base_urls
            .services = server.base_url();
        crate::profiles::save_profiles(&paths, &profiles).expect("save");

        let result = search_contacts(
            &paths,
            Some("default"),
            None,
            ContactSearchOptions {
                query: Some("John".to_owned()),
                email: Some("john@example.com".to_owned()),
                phone: None,
                limit: 25,
                start_after_id: None,
                start_after: None,
            },
        )
        .expect("contacts");

        mock.assert();
        assert_eq!(result.location_id, "loc_123");
        assert_eq!(result.email_filter, Some("john@example.com".to_owned()));
        assert_eq!(
            result.body_json.unwrap()["contacts"][0]["token"],
            "[REDACTED]"
        );
    }

    #[test]
    fn contact_search_dry_run_uses_location_override_without_profile() {
        let temp = tempfile::tempdir().expect("tempdir");
        let paths = resolve_paths(Some(temp.path()));

        let result = contacts_search_dry_run(
            &paths,
            Some("missing"),
            Some("loc_override"),
            ContactSearchOptions {
                query: Some("John".to_owned()),
                email: None,
                phone: Some("+15551234567".to_owned()),
                limit: 10,
                start_after_id: None,
                start_after: None,
            },
        )
        .expect("dry run");

        assert_eq!(result.method, "POST");
        assert_eq!(result.path, "/contacts/search");
        assert_eq!(result.location_id, "loc_override");
        assert_eq!(
            result.context.location_id.unwrap().source,
            crate::ContextSource::Override
        );
        assert_eq!(result.request_body_json["locationId"], "loc_override");
        assert_eq!(result.request_body_json["filters"][0]["field"], "phone");
        assert_eq!(result.request_body_json["filters"][0]["operator"], "eq");
        assert_eq!(
            result.request_body_json["filters"][0]["value"],
            "+15551234567"
        );
    }

    #[test]
    fn contact_search_requires_query_or_exact_filter() {
        let temp = tempfile::tempdir().expect("tempdir");
        let paths = resolve_paths(Some(temp.path()));

        let error = contacts_search_dry_run(
            &paths,
            None,
            Some("loc_123"),
            ContactSearchOptions {
                query: None,
                email: None,
                phone: None,
                limit: 25,
                start_after_id: None,
                start_after: None,
            },
        )
        .expect_err("missing search term");

        assert_eq!(error.code(), "validation_error");
    }

    #[test]
    fn contact_get_rejects_path_injection() {
        let temp = tempfile::tempdir().expect("tempdir");
        let paths = resolve_paths(Some(temp.path()));

        let error = get_contact_dry_run(&paths, None, Some("loc_123"), "../contact_123")
            .expect_err("invalid id");

        assert_eq!(error.code(), "validation_error");
    }

    #[test]
    fn contact_create_dry_run_redacts_pii_and_writes_audit() {
        let temp = tempfile::tempdir().expect("tempdir");
        let paths = resolve_paths(Some(temp.path()));

        let result = create_contact_dry_run(
            &paths,
            None,
            Some("loc_123"),
            ContactCreateOptions {
                fields: contact_fields(),
                idempotency_key: Some("create-contact-1".to_owned()),
            },
        )
        .expect("dry run");

        assert_eq!(result.method, "POST");
        assert_eq!(result.path, "/contacts/");
        assert_eq!(result.request_body_json["locationId"], "loc_123");
        assert_eq!(result.request_body_json["email"], "[REDACTED]");
        assert_eq!(result.request_body_json["phone"], "[REDACTED]");
        assert_eq!(result.preflight.status, "planned");
        let audit =
            std::fs::read_to_string(paths.audit_dir.join("audit.jsonl")).expect("audit journal");
        assert!(audit.contains("contacts.create"));
        assert!(!audit.contains("john@example.com"));
        assert!(!audit.contains("+15551234567"));
    }

    #[test]
    fn contact_create_posts_after_policy_idempotency_and_duplicate_preflight() {
        let server = MockServer::start();
        let duplicate = server.mock(|when, then| {
            when.method(POST)
                .path("/contacts/search")
                .header("authorization", "Bearer pit-secret")
                .json_body(json!({
                    "locationId": "loc_123",
                    "pageLimit": 5,
                    "filters": [
                        { "field": "email", "operator": "eq", "value": "john@example.com" }
                    ]
                }));
            then.status(200)
                .header("content-type", "application/json")
                .json_body(json!({ "contacts": [], "total": 0 }));
        });
        let create = server.mock(|when, then| {
            when.method(POST)
                .path("/contacts/")
                .header("authorization", "Bearer pit-secret")
                .json_body(json!({
                    "locationId": "loc_123",
                    "firstName": "John",
                    "email": "john@example.com",
                    "tags": ["cli-smoke"]
                }));
            then.status(201)
                .header("content-type", "application/json")
                .json_body(json!({
                    "contact": {
                        "id": "contact_123",
                        "email": "john@example.com"
                    }
                }));
        });
        let temp = tempfile::tempdir().expect("tempdir");
        let paths = resolve_paths(Some(temp.path()));
        add_pit(
            &paths,
            "default",
            "pit-secret".to_owned(),
            Some("loc_123".to_owned()),
            None,
            true,
        )
        .expect("pit");
        let mut profiles = crate::profiles::load_profiles(&paths).expect("profiles");
        let profile = profiles.profiles.get_mut("default").expect("profile");
        profile.base_urls.services = server.base_url();
        profile.policy.allow_destructive = true;
        crate::profiles::save_profiles(&paths, &profiles).expect("save");

        let result = create_contact(
            &paths,
            Some("default"),
            None,
            ContactCreateOptions {
                fields: ContactWriteFields {
                    first_name: Some("John".to_owned()),
                    email: Some("john@example.com".to_owned()),
                    tags: vec!["cli-smoke".to_owned()],
                    ..ContactWriteFields::default()
                },
                idempotency_key: Some("create-contact-1".to_owned()),
            },
        )
        .expect("create contact");

        duplicate.assert();
        create.assert();
        assert!(result.success);
        assert_eq!(result.contact_id, Some("contact_123".to_owned()));
        assert_eq!(result.preflight.status, "passed");
        assert_eq!(result.body_json.unwrap()["contact"]["email"], "[REDACTED]");
        let idempotency = crate::list_idempotency_records(&paths).expect("idempotency");
        assert_eq!(idempotency.count, 1);
        assert_eq!(
            idempotency.records[0].resource_id,
            Some("contact_123".to_owned())
        );
    }

    #[test]
    fn contact_create_duplicate_preflight_blocks_existing_contact() {
        let server = MockServer::start();
        let duplicate = server.mock(|when, then| {
            when.method(POST).path("/contacts/search");
            then.status(200)
                .header("content-type", "application/json")
                .json_body(json!({
                    "contacts": [{ "id": "contact_existing" }],
                    "total": 1
                }));
        });
        let temp = tempfile::tempdir().expect("tempdir");
        let paths = resolve_paths(Some(temp.path()));
        add_pit(
            &paths,
            "default",
            "pit-secret".to_owned(),
            Some("loc_123".to_owned()),
            None,
            true,
        )
        .expect("pit");
        let mut profiles = crate::profiles::load_profiles(&paths).expect("profiles");
        let profile = profiles.profiles.get_mut("default").expect("profile");
        profile.base_urls.services = server.base_url();
        profile.policy.allow_destructive = true;
        crate::profiles::save_profiles(&paths, &profiles).expect("save");

        let error = create_contact(
            &paths,
            Some("default"),
            None,
            ContactCreateOptions {
                fields: ContactWriteFields {
                    email: Some("john@example.com".to_owned()),
                    ..ContactWriteFields::default()
                },
                idempotency_key: Some("create-contact-1".to_owned()),
            },
        )
        .expect_err("duplicate");

        duplicate.assert();
        assert_eq!(error.code(), "validation_error");
        assert!(error.to_string().contains("contact_existing"));
    }

    #[test]
    fn contact_update_puts_after_policy_and_idempotency() {
        let server = MockServer::start();
        let update = server.mock(|when, then| {
            when.method(PUT)
                .path("/contacts/contact_123")
                .header("authorization", "Bearer pit-secret")
                .json_body(json!({
                    "firstName": "Jane",
                    "phone": "+15557654321"
                }));
            then.status(200)
                .header("content-type", "application/json")
                .json_body(json!({
                    "contact": {
                        "id": "contact_123",
                        "phone": "+15557654321"
                    }
                }));
        });
        let temp = tempfile::tempdir().expect("tempdir");
        let paths = resolve_paths(Some(temp.path()));
        add_pit(
            &paths,
            "default",
            "pit-secret".to_owned(),
            Some("loc_123".to_owned()),
            None,
            true,
        )
        .expect("pit");
        let mut profiles = crate::profiles::load_profiles(&paths).expect("profiles");
        let profile = profiles.profiles.get_mut("default").expect("profile");
        profile.base_urls.services = server.base_url();
        profile.policy.allow_destructive = true;
        crate::profiles::save_profiles(&paths, &profiles).expect("save");

        let result = update_contact(
            &paths,
            Some("default"),
            None,
            ContactUpdateOptions {
                contact_id: "contact_123".to_owned(),
                fields: ContactWriteFields {
                    first_name: Some("Jane".to_owned()),
                    phone: Some("+15557654321".to_owned()),
                    ..ContactWriteFields::default()
                },
                idempotency_key: Some("update-contact-1".to_owned()),
            },
        )
        .expect("update contact");

        update.assert();
        assert!(result.success);
        assert_eq!(result.contact_id, Some("contact_123".to_owned()));
        assert_eq!(result.body_json.unwrap()["contact"]["phone"], "[REDACTED]");
    }

    fn contact_fields() -> ContactWriteFields {
        ContactWriteFields {
            first_name: Some("John".to_owned()),
            email: Some("john@example.com".to_owned()),
            phone: Some("+15551234567".to_owned()),
            tags: vec!["cli-smoke".to_owned()],
            ..ContactWriteFields::default()
        }
    }
}
