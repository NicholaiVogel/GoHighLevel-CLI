use serde::{Deserialize, Serialize};
use serde_json::{Map, Value, json};

use crate::audit::{
    AuditEntryInput, AuditResource, AuditResultSummary, AuditUpstreamSummary, append_audit_entry,
};
use crate::client::{
    AuthClass, RawDeleteRequest, RawGetRequest, RawPostJsonRequest, RawPutJsonRequest, delete,
    post_json, put_json, raw_get,
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

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AppointmentNotesListOptions {
    pub appointment_id: String,
    pub limit: u32,
    pub offset: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AppointmentNoteWriteOptions {
    pub appointment_id: String,
    pub note_id: Option<String>,
    pub body: String,
    pub user_id: Option<String>,
    pub idempotency_key: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AppointmentNoteDeleteOptions {
    pub appointment_id: String,
    pub note_id: String,
    pub idempotency_key: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AppointmentNotesListDryRun {
    pub method: &'static str,
    pub surface: &'static str,
    pub path: String,
    pub context: ResolvedContext,
    pub appointment_id: String,
    pub limit: u32,
    pub offset: u32,
    pub auth_class: &'static str,
    pub network: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AppointmentNotesListResult {
    pub profile: String,
    pub context: ResolvedContext,
    pub appointment_id: String,
    pub endpoint: String,
    pub url: String,
    pub status: u16,
    pub success: bool,
    pub note_count: usize,
    pub note_ids: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub has_more: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub body_json: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub body_text: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AppointmentNoteWriteDryRun {
    pub method: &'static str,
    pub surface: &'static str,
    pub path: String,
    pub context: ResolvedContext,
    pub appointment_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub note_id: Option<String>,
    pub request_body_json: Value,
    pub request_hash: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub idempotency_key: Option<String>,
    pub audit_entry_id: String,
    pub auth_class: &'static str,
    pub network: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AppointmentNoteDeleteDryRun {
    pub method: &'static str,
    pub surface: &'static str,
    pub path: String,
    pub context: ResolvedContext,
    pub appointment_id: String,
    pub note_id: String,
    pub request_hash: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub idempotency_key: Option<String>,
    pub audit_entry_id: String,
    pub auth_class: &'static str,
    pub network: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AppointmentNoteWriteResult {
    pub profile: String,
    pub context: ResolvedContext,
    pub endpoint: String,
    pub url: String,
    pub status: u16,
    pub success: bool,
    pub replayed: bool,
    pub appointment_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub note_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub idempotency_key: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub idempotency_state: Option<String>,
    pub request_hash: String,
    pub audit_entry_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub body_json: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub body_text: Option<String>,
}

pub fn appointment_notes_list_dry_run(
    paths: &crate::ConfigPaths,
    profile_name: Option<&str>,
    location_override: Option<&str>,
    options: AppointmentNotesListOptions,
) -> Result<AppointmentNotesListDryRun> {
    validate_notes_list_options(&options)?;
    let context = resolve_context_for_dry_run(paths, profile_name, location_override, None)?;
    Ok(AppointmentNotesListDryRun {
        method: "GET",
        surface: "services",
        path: notes_list_endpoint(&options.appointment_id, options.limit, options.offset),
        context,
        appointment_id: options.appointment_id,
        limit: options.limit,
        offset: options.offset,
        auth_class: "pit",
        network: false,
    })
}

pub fn list_appointment_notes(
    paths: &crate::ConfigPaths,
    profile_name: Option<&str>,
    location_override: Option<&str>,
    options: AppointmentNotesListOptions,
) -> Result<AppointmentNotesListResult> {
    validate_notes_list_options(&options)?;
    let context = resolve_context(paths, profile_name, location_override, None)?;
    let endpoint = notes_list_endpoint(&options.appointment_id, options.limit, options.offset);
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
    let note_ids = extract_note_ids(response.body_json.as_ref());
    let note_count = note_ids.len();
    let body_json = response.body_json.as_ref().map(redact_json);
    let has_more = body_json
        .as_ref()
        .and_then(|body| body.get("hasMore").and_then(Value::as_bool));

    Ok(AppointmentNotesListResult {
        profile: context.profile.clone(),
        context,
        appointment_id: options.appointment_id,
        endpoint,
        url: response.url,
        status: response.status,
        success: response.success,
        note_count,
        note_ids,
        has_more,
        body_json,
        body_text: response.body_text,
    })
}

pub fn create_appointment_note_dry_run(
    paths: &crate::ConfigPaths,
    profile_name: Option<&str>,
    location_override: Option<&str>,
    options: AppointmentNoteWriteOptions,
) -> Result<AppointmentNoteWriteDryRun> {
    note_write_dry_run(
        paths,
        profile_name,
        location_override,
        options,
        "appointments.notes.create",
        "POST",
    )
}

pub fn update_appointment_note_dry_run(
    paths: &crate::ConfigPaths,
    profile_name: Option<&str>,
    location_override: Option<&str>,
    options: AppointmentNoteWriteOptions,
) -> Result<AppointmentNoteWriteDryRun> {
    note_write_dry_run(
        paths,
        profile_name,
        location_override,
        options,
        "appointments.notes.update",
        "PUT",
    )
}

fn note_write_dry_run(
    paths: &crate::ConfigPaths,
    profile_name: Option<&str>,
    location_override: Option<&str>,
    options: AppointmentNoteWriteOptions,
    command: &str,
    method: &'static str,
) -> Result<AppointmentNoteWriteDryRun> {
    validate_note_write_options(&options, method == "PUT")?;
    let context = resolve_context_for_dry_run(paths, profile_name, location_override, None)?;
    let endpoint = note_write_endpoint(&options.appointment_id, options.note_id.as_deref(), method);
    let request_body_json = note_body(&options);
    let request_hash = stable_request_hash(&json!({
        "method": method,
        "path": endpoint,
        "body": request_body_json,
    }))?;
    let audit = append_audit_entry(
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
                resource_type: "appointment_note".to_owned(),
                id: options.note_id.clone(),
            }),
            request_summary: json!({
                "endpoint": endpoint,
                "request_body": redact_json(&request_body_json),
                "request_hash": request_hash,
            }),
            upstream: None,
            result: AuditResultSummary {
                status: "dry_run".to_owned(),
                resource_id: options.note_id.clone(),
                message: Some(
                    "appointment note write dry-run; no network mutation performed".to_owned(),
                ),
            },
            error: None,
        },
    )?;

    Ok(AppointmentNoteWriteDryRun {
        method,
        surface: "services",
        path: endpoint,
        context,
        appointment_id: options.appointment_id,
        note_id: options.note_id,
        request_body_json: redact_json(&request_body_json),
        request_hash,
        idempotency_key: options.idempotency_key,
        audit_entry_id: audit.id,
        auth_class: "pit",
        network: false,
    })
}

pub fn create_appointment_note(
    paths: &crate::ConfigPaths,
    profile_name: Option<&str>,
    location_override: Option<&str>,
    options: AppointmentNoteWriteOptions,
) -> Result<AppointmentNoteWriteResult> {
    note_write(
        paths,
        profile_name,
        location_override,
        options,
        "appointments.notes.create",
        "POST",
    )
}

pub fn update_appointment_note(
    paths: &crate::ConfigPaths,
    profile_name: Option<&str>,
    location_override: Option<&str>,
    options: AppointmentNoteWriteOptions,
) -> Result<AppointmentNoteWriteResult> {
    note_write(
        paths,
        profile_name,
        location_override,
        options,
        "appointments.notes.update",
        "PUT",
    )
}

fn note_write(
    paths: &crate::ConfigPaths,
    profile_name: Option<&str>,
    location_override: Option<&str>,
    options: AppointmentNoteWriteOptions,
    command: &str,
    method: &'static str,
) -> Result<AppointmentNoteWriteResult> {
    validate_note_write_options(&options, method == "PUT")?;
    let context = resolve_context(paths, profile_name, location_override, None)?;
    ensure_write_allowed(paths, &context.profile)?;
    let endpoint = note_write_endpoint(&options.appointment_id, options.note_id.as_deref(), method);
    let request_body = note_body(&options);
    let request_hash = stable_request_hash(&json!({
        "method": method,
        "path": endpoint,
        "body": request_body,
    }))?;
    let idempotency_key = options
        .idempotency_key
        .clone()
        .ok_or_else(|| GhlError::Validation {
            message: format!(
                "real appointment note {} requires --idempotency-key <key>",
                if method == "POST" { "create" } else { "update" }
            ),
        })?;

    if let Some(replay) = maybe_replay(
        paths,
        &context,
        command,
        &idempotency_key,
        &request_hash,
        &endpoint,
        options.note_id.clone(),
    )? {
        return Ok(replay);
    }

    record_in_progress(
        paths,
        &context,
        command,
        &idempotency_key,
        &request_hash,
        options.note_id.clone(),
    )?;
    let response = if method == "POST" {
        post_json(
            paths,
            profile_name,
            RawPostJsonRequest {
                surface: Surface::Services,
                path: endpoint.clone(),
                auth_class: AuthClass::Pit,
                body: request_body.clone(),
                include_body: true,
            },
        )?
    } else {
        put_json(
            paths,
            profile_name,
            RawPutJsonRequest {
                surface: Surface::Services,
                path: endpoint.clone(),
                auth_class: AuthClass::Pit,
                body: request_body.clone(),
                include_body: true,
            },
        )?
    };
    let note_id = extract_note_id(response.body_json.as_ref()).or(options.note_id.clone());
    let audit = write_result_audit(
        paths,
        &context,
        command,
        &endpoint,
        Some(request_body),
        &request_hash,
        &idempotency_key,
        note_id.clone(),
        response.status,
        response.headers.get("x-request-id").cloned(),
        response.success,
        response
            .body_text
            .clone()
            .or_else(|| response.body_json.as_ref().map(Value::to_string)),
    )?;
    record_done(
        paths,
        &context,
        IdempotencyDone {
            command,
            key: &idempotency_key,
            request_hash: &request_hash,
            note_id: note_id.clone(),
            audit_entry_id: Some(audit.id.clone()),
            success: response.success,
        },
    )?;

    Ok(AppointmentNoteWriteResult {
        profile: context.profile.clone(),
        context,
        endpoint,
        url: response.url,
        status: response.status,
        success: response.success,
        replayed: false,
        appointment_id: options.appointment_id,
        note_id,
        idempotency_key: Some(idempotency_key),
        idempotency_state: Some("recorded".to_owned()),
        request_hash,
        audit_entry_id: audit.id,
        body_json: response.body_json.map(|body| redact_json(&body)),
        body_text: response.body_text,
    })
}

pub fn delete_appointment_note_dry_run(
    paths: &crate::ConfigPaths,
    profile_name: Option<&str>,
    location_override: Option<&str>,
    options: AppointmentNoteDeleteOptions,
) -> Result<AppointmentNoteDeleteDryRun> {
    validate_note_delete_options(&options)?;
    let context = resolve_context_for_dry_run(paths, profile_name, location_override, None)?;
    let endpoint = note_id_endpoint(&options.appointment_id, &options.note_id);
    let request_hash = stable_request_hash(&json!({ "method": "DELETE", "path": endpoint }))?;
    let audit = append_audit_entry(
        paths,
        AuditEntryInput {
            profile: Some(context.profile.clone()),
            company_id: context.company_id.as_ref().map(|value| value.value.clone()),
            location_id: context
                .location_id
                .as_ref()
                .map(|value| value.value.clone()),
            command: "appointments.notes.delete".to_owned(),
            action_class: "sensitive_dry_run".to_owned(),
            dry_run: true,
            policy_flags: write_policy_flags(),
            resource: Some(AuditResource {
                resource_type: "appointment_note".to_owned(),
                id: Some(options.note_id.clone()),
            }),
            request_summary: json!({
                "endpoint": endpoint,
                "request_hash": request_hash,
            }),
            upstream: None,
            result: AuditResultSummary {
                status: "dry_run".to_owned(),
                resource_id: Some(options.note_id.clone()),
                message: Some(
                    "appointment note delete dry-run; no network mutation performed".to_owned(),
                ),
            },
            error: None,
        },
    )?;

    Ok(AppointmentNoteDeleteDryRun {
        method: "DELETE",
        surface: "services",
        path: endpoint,
        context,
        appointment_id: options.appointment_id,
        note_id: options.note_id,
        request_hash,
        idempotency_key: options.idempotency_key,
        audit_entry_id: audit.id,
        auth_class: "pit",
        network: false,
    })
}

pub fn delete_appointment_note(
    paths: &crate::ConfigPaths,
    profile_name: Option<&str>,
    location_override: Option<&str>,
    options: AppointmentNoteDeleteOptions,
) -> Result<AppointmentNoteWriteResult> {
    validate_note_delete_options(&options)?;
    let context = resolve_context(paths, profile_name, location_override, None)?;
    ensure_write_allowed(paths, &context.profile)?;
    let endpoint = note_id_endpoint(&options.appointment_id, &options.note_id);
    let request_hash = stable_request_hash(&json!({ "method": "DELETE", "path": endpoint }))?;
    let idempotency_key = options
        .idempotency_key
        .clone()
        .ok_or_else(|| GhlError::Validation {
            message: "real appointment note delete requires --idempotency-key <key>".to_owned(),
        })?;

    if let Some(replay) = maybe_replay(
        paths,
        &context,
        "appointments.notes.delete",
        &idempotency_key,
        &request_hash,
        &endpoint,
        Some(options.note_id.clone()),
    )? {
        return Ok(replay);
    }

    record_in_progress(
        paths,
        &context,
        "appointments.notes.delete",
        &idempotency_key,
        &request_hash,
        Some(options.note_id.clone()),
    )?;
    let response = delete(
        paths,
        profile_name,
        RawDeleteRequest {
            surface: Surface::Services,
            path: endpoint.clone(),
            auth_class: AuthClass::Pit,
            include_body: true,
        },
    )?;
    let audit = write_result_audit(
        paths,
        &context,
        "appointments.notes.delete",
        &endpoint,
        None,
        &request_hash,
        &idempotency_key,
        Some(options.note_id.clone()),
        response.status,
        response.headers.get("x-request-id").cloned(),
        response.success,
        response
            .body_text
            .clone()
            .or_else(|| response.body_json.as_ref().map(Value::to_string)),
    )?;
    record_done(
        paths,
        &context,
        IdempotencyDone {
            command: "appointments.notes.delete",
            key: &idempotency_key,
            request_hash: &request_hash,
            note_id: Some(options.note_id.clone()),
            audit_entry_id: Some(audit.id.clone()),
            success: response.success,
        },
    )?;

    Ok(AppointmentNoteWriteResult {
        profile: context.profile.clone(),
        context,
        endpoint,
        url: response.url,
        status: response.status,
        success: response.success,
        replayed: false,
        appointment_id: options.appointment_id,
        note_id: Some(options.note_id),
        idempotency_key: Some(idempotency_key),
        idempotency_state: Some("recorded".to_owned()),
        request_hash,
        audit_entry_id: audit.id,
        body_json: response.body_json.map(|body| redact_json(&body)),
        body_text: response.body_text,
    })
}

#[allow(clippy::too_many_arguments)]
fn write_result_audit(
    paths: &crate::ConfigPaths,
    context: &ResolvedContext,
    command: &str,
    endpoint: &str,
    request_body: Option<Value>,
    request_hash: &str,
    idempotency_key: &str,
    note_id: Option<String>,
    status_code: u16,
    request_id: Option<String>,
    success: bool,
    error: Option<String>,
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
                resource_type: "appointment_note".to_owned(),
                id: note_id.clone(),
            }),
            request_summary: json!({
                "endpoint": endpoint,
                "request_body": request_body.map(|body| redact_json(&body)),
                "request_hash": request_hash,
                "idempotency_key": idempotency_key,
            }),
            upstream: Some(AuditUpstreamSummary {
                request_id,
                status_code: Some(status_code),
                endpoint_key: Some(command.to_owned()),
            }),
            result: AuditResultSummary {
                status: if success { "success" } else { "failed" }.to_owned(),
                resource_id: note_id,
                message: None,
            },
            error,
        },
    )
}

fn maybe_replay(
    paths: &crate::ConfigPaths,
    context: &ResolvedContext,
    command: &str,
    idempotency_key: &str,
    request_hash: &str,
    endpoint: &str,
    fallback_note_id: Option<String>,
) -> Result<Option<AppointmentNoteWriteResult>> {
    let idempotency = check_idempotency_key(
        paths,
        &context.profile,
        context
            .location_id
            .as_ref()
            .map(|value| value.value.as_str()),
        command,
        idempotency_key,
        request_hash,
    )?;
    if idempotency.state != IdempotencyCheckState::Replay {
        return Ok(None);
    }
    let existing = idempotency.existing.expect("existing replay record");
    let note_id = existing.resource_id.clone().or(fallback_note_id);
    let audit = append_audit_entry(
        paths,
        AuditEntryInput {
            profile: Some(context.profile.clone()),
            company_id: context.company_id.as_ref().map(|value| value.value.clone()),
            location_id: context
                .location_id
                .as_ref()
                .map(|value| value.value.clone()),
            command: command.to_owned(),
            action_class: "idempotency_replay".to_owned(),
            dry_run: false,
            policy_flags: write_policy_flags(),
            resource: Some(AuditResource {
                resource_type: "appointment_note".to_owned(),
                id: note_id.clone(),
            }),
            request_summary: json!({
                "endpoint": endpoint,
                "idempotency_key": idempotency_key,
                "request_hash": request_hash,
            }),
            upstream: None,
            result: AuditResultSummary {
                status: "replayed".to_owned(),
                resource_id: note_id.clone(),
                message: Some(
                    "idempotency key matched a previous request; no network mutation performed"
                        .to_owned(),
                ),
            },
            error: None,
        },
    )?;
    Ok(Some(AppointmentNoteWriteResult {
        profile: context.profile.clone(),
        context: context.clone(),
        endpoint: endpoint.to_owned(),
        url: String::new(),
        status: 200,
        success: existing.status == IdempotencyStatus::Succeeded,
        replayed: true,
        appointment_id: appointment_id_from_endpoint(endpoint),
        note_id,
        idempotency_key: Some(idempotency_key.to_owned()),
        idempotency_state: Some("replay".to_owned()),
        request_hash: request_hash.to_owned(),
        audit_entry_id: audit.id,
        body_json: None,
        body_text: None,
    }))
}

fn record_in_progress(
    paths: &crate::ConfigPaths,
    context: &ResolvedContext,
    command: &str,
    key: &str,
    request_hash: &str,
    note_id: Option<String>,
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
            resource_id: note_id,
            audit_entry_id: None,
        },
    )?;
    Ok(())
}

struct IdempotencyDone<'a> {
    command: &'a str,
    key: &'a str,
    request_hash: &'a str,
    note_id: Option<String>,
    audit_entry_id: Option<String>,
    success: bool,
}

fn record_done(
    paths: &crate::ConfigPaths,
    context: &ResolvedContext,
    done: IdempotencyDone<'_>,
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
            resource_id: done.note_id,
            audit_entry_id: done.audit_entry_id,
        },
    )?;
    Ok(())
}

fn ensure_write_allowed(paths: &crate::ConfigPaths, profile_name: &str) -> Result<()> {
    let profiles = load_profiles(paths)?;
    let profile = profiles.get_required(profile_name)?;
    if !profile.policy.allow_destructive {
        return Err(GhlError::PolicyDenied {
            message: "profile policy blocks appointment note writes; enable allow_destructive for this profile before real appointment note mutations".to_owned(),
        });
    }
    Ok(())
}

fn notes_base_endpoint(appointment_id: &str) -> String {
    format!(
        "/calendars/events/appointments/{}/notes",
        appointment_id.trim()
    )
}

fn notes_list_endpoint(appointment_id: &str, limit: u32, offset: u32) -> String {
    format!(
        "{}?limit={limit}&offset={offset}",
        notes_base_endpoint(appointment_id)
    )
}

fn note_id_endpoint(appointment_id: &str, note_id: &str) -> String {
    format!("{}/{}", notes_base_endpoint(appointment_id), note_id.trim())
}

fn note_write_endpoint(appointment_id: &str, note_id: Option<&str>, method: &str) -> String {
    if method == "PUT" {
        note_id_endpoint(
            appointment_id,
            note_id.expect("note id validated for update"),
        )
    } else {
        notes_base_endpoint(appointment_id)
    }
}

fn note_body(options: &AppointmentNoteWriteOptions) -> Value {
    let mut body = Map::new();
    body.insert(
        "body".to_owned(),
        Value::String(options.body.trim().to_owned()),
    );
    if let Some(user_id) = options
        .user_id
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
    {
        body.insert("userId".to_owned(), Value::String(user_id.to_owned()));
    }
    Value::Object(body)
}

fn extract_note_ids(body: Option<&Value>) -> Vec<String> {
    let Some(body) = body else {
        return Vec::new();
    };
    body.get("notes")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(|note| note.get("id").and_then(Value::as_str).map(str::to_owned))
        .collect()
}

fn extract_note_id(body: Option<&Value>) -> Option<String> {
    let body = body?;
    body.get("id")
        .or_else(|| body.get("noteId"))
        .or_else(|| body.get("note").and_then(|note| note.get("id")))
        .and_then(Value::as_str)
        .map(str::to_owned)
}

fn appointment_id_from_endpoint(endpoint: &str) -> String {
    endpoint
        .trim_start_matches("/calendars/events/appointments/")
        .split('/')
        .next()
        .unwrap_or_default()
        .to_owned()
}

fn validate_notes_list_options(options: &AppointmentNotesListOptions) -> Result<()> {
    validate_path_segment(&options.appointment_id, "appointment id")?;
    if options.limit == 0 || options.limit > 100 {
        return Err(GhlError::Validation {
            message: "appointment notes --limit must be between 1 and 100".to_owned(),
        });
    }
    Ok(())
}

fn validate_note_write_options(
    options: &AppointmentNoteWriteOptions,
    require_note_id: bool,
) -> Result<()> {
    validate_path_segment(&options.appointment_id, "appointment id")?;
    if require_note_id {
        validate_path_segment(options.note_id.as_deref().unwrap_or_default(), "note id")?;
    }
    validate_optional_text(options.note_id.as_deref(), "note id")?;
    validate_optional_text(options.user_id.as_deref(), "user id")?;
    validate_optional_text(options.idempotency_key.as_deref(), "idempotency key")?;
    validate_note_body(&options.body)?;
    Ok(())
}

fn validate_note_delete_options(options: &AppointmentNoteDeleteOptions) -> Result<()> {
    validate_path_segment(&options.appointment_id, "appointment id")?;
    validate_path_segment(&options.note_id, "note id")?;
    validate_optional_text(options.idempotency_key.as_deref(), "idempotency key")?;
    Ok(())
}

fn validate_note_body(value: &str) -> Result<()> {
    if value.trim().is_empty() {
        return Err(GhlError::Validation {
            message: "appointment note body cannot be empty".to_owned(),
        });
    }
    if value.chars().any(|character| {
        character.is_ascii_control() && character != '\n' && character != '\r' && character != '\t'
    }) {
        return Err(GhlError::Validation {
            message: "appointment note body cannot contain control characters".to_owned(),
        });
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

fn write_policy_flags() -> Vec<String> {
    vec![
        "allow_destructive".to_owned(),
        "confirmation_required".to_owned(),
        "idempotency_key_required".to_owned(),
    ]
}

#[cfg(test)]
mod tests {
    use httpmock::Method::{DELETE, GET, POST, PUT};
    use httpmock::MockServer;
    use serde_json::json;

    use super::*;
    use crate::auth::add_pit;
    use crate::config::resolve_paths;

    #[test]
    fn appointment_notes_list_redacts_bodies_and_returns_ids() {
        let server = MockServer::start();
        let list = server.mock(|when, then| {
            when.method(GET)
                .path("/calendars/events/appointments/appt_123/notes")
                .query_param("limit", "10")
                .query_param("offset", "0");
            then.status(200).json_body(json!({
                "notes": [{ "id": "note_123", "body": "private note" }],
                "hasMore": false
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

        let result =
            list_appointment_notes(&paths, Some("default"), None, list_options()).expect("list");

        list.assert();
        assert_eq!(result.note_ids, vec!["note_123".to_owned()]);
        assert_eq!(result.body_json.unwrap()["notes"][0]["body"], "[REDACTED]");
    }

    #[test]
    fn appointment_note_create_dry_run_redacts_body_in_output_and_audit() {
        let temp = tempfile::tempdir().expect("tempdir");
        let paths = resolve_paths(Some(temp.path()));

        let result = create_appointment_note_dry_run(
            &paths,
            None,
            Some("loc_123"),
            write_options(None, Some("key-1")),
        )
        .expect("dry run");

        assert_eq!(result.method, "POST");
        assert_eq!(result.request_body_json["body"], "[REDACTED]");
        let audit = std::fs::read_to_string(crate::audit_journal_path(&paths)).expect("audit");
        assert!(audit.contains("appointments.notes.create"));
        assert!(!audit.contains("private note"));
    }

    #[test]
    fn appointment_note_create_posts_after_policy_and_idempotency() {
        let server = MockServer::start();
        let create = server.mock(|when, then| {
            when.method(POST)
                .path("/calendars/events/appointments/appt_123/notes")
                .header("authorization", "Bearer pit-secret")
                .json_body(json!({ "body": "private note", "userId": "user_123" }));
            then.status(201)
                .json_body(json!({ "note": { "id": "note_123", "body": "private note" } }));
        });
        let temp = tempfile::tempdir().expect("tempdir");
        let paths = writable_paths(temp.path(), &server);

        let result = create_appointment_note(
            &paths,
            Some("default"),
            None,
            write_options(None, Some("create-note-1")),
        )
        .expect("create");

        create.assert();
        assert!(result.success);
        assert_eq!(result.note_id, Some("note_123".to_owned()));
        assert_eq!(result.body_json.unwrap()["note"]["body"], "[REDACTED]");
    }

    #[test]
    fn appointment_note_update_puts_after_policy_and_idempotency() {
        let server = MockServer::start();
        let update = server.mock(|when, then| {
            when.method(PUT)
                .path("/calendars/events/appointments/appt_123/notes/note_123")
                .json_body(json!({ "body": "private note", "userId": "user_123" }));
            then.status(200)
                .json_body(json!({ "note": { "id": "note_123", "body": "private note" } }));
        });
        let temp = tempfile::tempdir().expect("tempdir");
        let paths = writable_paths(temp.path(), &server);

        let result = update_appointment_note(
            &paths,
            Some("default"),
            None,
            write_options(Some("note_123"), Some("update-note-1")),
        )
        .expect("update");

        update.assert();
        assert!(result.success);
        assert_eq!(result.note_id, Some("note_123".to_owned()));
    }

    #[test]
    fn appointment_note_delete_deletes_after_policy_and_idempotency() {
        let server = MockServer::start();
        let delete_mock = server.mock(|when, then| {
            when.method(DELETE)
                .path("/calendars/events/appointments/appt_123/notes/note_123");
            then.status(200).json_body(json!({ "success": true }));
        });
        let temp = tempfile::tempdir().expect("tempdir");
        let paths = writable_paths(temp.path(), &server);

        let result = delete_appointment_note(
            &paths,
            Some("default"),
            None,
            delete_options(Some("delete-note-1")),
        )
        .expect("delete");

        delete_mock.assert();
        assert!(result.success);
        assert_eq!(result.note_id, Some("note_123".to_owned()));
    }

    fn writable_paths(temp: &std::path::Path, server: &MockServer) -> crate::ConfigPaths {
        let paths = resolve_paths(Some(temp));
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
        paths
    }

    fn list_options() -> AppointmentNotesListOptions {
        AppointmentNotesListOptions {
            appointment_id: "appt_123".to_owned(),
            limit: 10,
            offset: 0,
        }
    }

    fn write_options(
        note_id: Option<&str>,
        idempotency_key: Option<&str>,
    ) -> AppointmentNoteWriteOptions {
        AppointmentNoteWriteOptions {
            appointment_id: "appt_123".to_owned(),
            note_id: note_id.map(str::to_owned),
            body: "private note".to_owned(),
            user_id: Some("user_123".to_owned()),
            idempotency_key: idempotency_key.map(str::to_owned),
        }
    }

    fn delete_options(idempotency_key: Option<&str>) -> AppointmentNoteDeleteOptions {
        AppointmentNoteDeleteOptions {
            appointment_id: "appt_123".to_owned(),
            note_id: "note_123".to_owned(),
            idempotency_key: idempotency_key.map(str::to_owned),
        }
    }
}
