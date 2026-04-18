use serde::{Deserialize, Serialize};
use serde_json::{Map, Value, json};
use url::form_urlencoded::Serializer;

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
use crate::surfaces::Surface;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum OpportunityStatus {
    Open,
    Won,
    Lost,
    Abandoned,
    All,
}

impl OpportunityStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Open => "open",
            Self::Won => "won",
            Self::Lost => "lost",
            Self::Abandoned => "abandoned",
            Self::All => "all",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct OpportunitySearchOptions {
    pub query: Option<String>,
    pub pipeline_id: Option<String>,
    pub pipeline_stage_id: Option<String>,
    pub contact_id: Option<String>,
    pub status: Option<OpportunityStatus>,
    pub assigned_to: Option<String>,
    pub limit: u32,
    pub page: Option<u32>,
    pub start_after_id: Option<String>,
    pub start_after: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct OpportunityCreateOptions {
    pub name: String,
    pub pipeline_id: String,
    pub pipeline_stage_id: Option<String>,
    pub contact_id: String,
    pub status: OpportunityStatus,
    pub monetary_value: Option<f64>,
    pub assigned_to: Option<String>,
    pub idempotency_key: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct OpportunityUpdateOptions {
    pub opportunity_id: String,
    pub name: Option<String>,
    pub pipeline_id: Option<String>,
    pub pipeline_stage_id: Option<String>,
    pub status: Option<OpportunityStatus>,
    pub monetary_value: Option<f64>,
    pub assigned_to: Option<String>,
    pub idempotency_key: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct OpportunityDuplicatePreflight {
    pub status: String,
    pub checked: bool,
    pub duplicate_count: usize,
    pub duplicate_opportunity_ids: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct OpportunityWriteDryRun {
    pub method: &'static str,
    pub surface: &'static str,
    pub path: String,
    pub context: ResolvedContext,
    pub location_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub opportunity_id: Option<String>,
    pub request_body_json: Value,
    pub request_hash: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub idempotency_key: Option<String>,
    pub preflight: OpportunityDuplicatePreflight,
    pub audit_entry_id: String,
    pub auth_class: &'static str,
    pub network: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct OpportunityWriteResult {
    pub profile: String,
    pub context: ResolvedContext,
    pub location_id: String,
    pub endpoint: String,
    pub url: String,
    pub status: u16,
    pub success: bool,
    pub replayed: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub opportunity_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub idempotency_key: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub idempotency_state: Option<String>,
    pub request_hash: String,
    pub preflight: OpportunityDuplicatePreflight,
    pub audit_entry_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub body_json: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub body_text: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct OpportunitySearchResult {
    pub profile: String,
    pub context: ResolvedContext,
    pub location_id: String,
    pub endpoint: String,
    pub url: String,
    pub status: u16,
    pub success: bool,
    pub limit: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub opportunity_status: Option<OpportunityStatus>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub body_json: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub body_text: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct OpportunitySearchDryRun {
    pub method: &'static str,
    pub surface: &'static str,
    pub path: String,
    pub context: ResolvedContext,
    pub location_id: String,
    pub limit: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub opportunity_status: Option<OpportunityStatus>,
    pub auth_class: &'static str,
    pub network: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct OpportunityGetResult {
    pub profile: String,
    pub context: ResolvedContext,
    pub location_id: String,
    pub opportunity_id: String,
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
pub struct OpportunityGetDryRun {
    pub method: &'static str,
    pub surface: &'static str,
    pub path: String,
    pub context: ResolvedContext,
    pub location_id: String,
    pub opportunity_id: String,
    pub auth_class: &'static str,
    pub network: bool,
}

pub fn search_opportunities(
    paths: &crate::ConfigPaths,
    profile_name: Option<&str>,
    location_override: Option<&str>,
    options: OpportunitySearchOptions,
) -> Result<OpportunitySearchResult> {
    validate_search_options(&options)?;
    let context = resolve_context(paths, profile_name, location_override, None)?;
    let location_id = context.require_location_id()?.to_owned();
    let endpoint = opportunities_search_endpoint(&location_id, &options);
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

    Ok(OpportunitySearchResult {
        profile: context.profile.clone(),
        context,
        location_id,
        endpoint,
        url: response.url,
        status: response.status,
        success: response.success,
        limit: options.limit,
        opportunity_status: options.status,
        body_json: response.body_json,
        body_text: response.body_text,
    })
}

pub fn get_opportunity(
    paths: &crate::ConfigPaths,
    profile_name: Option<&str>,
    location_override: Option<&str>,
    opportunity_id: &str,
) -> Result<OpportunityGetResult> {
    validate_opportunity_id(opportunity_id)?;
    let context = resolve_context(paths, profile_name, location_override, None)?;
    let location_id = context.require_location_id()?.to_owned();
    let endpoint = opportunity_endpoint(opportunity_id);
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

    Ok(OpportunityGetResult {
        profile: context.profile.clone(),
        context,
        location_id,
        opportunity_id: opportunity_id.to_owned(),
        endpoint,
        url: response.url,
        status: response.status,
        success: response.success,
        body_json: response.body_json,
        body_text: response.body_text,
    })
}

pub fn opportunities_search_dry_run(
    paths: &crate::ConfigPaths,
    profile_name: Option<&str>,
    location_override: Option<&str>,
    options: OpportunitySearchOptions,
) -> Result<OpportunitySearchDryRun> {
    validate_search_options(&options)?;
    let context = resolve_context_for_dry_run(paths, profile_name, location_override, None)?;
    let location_id = context.require_location_id()?.to_owned();

    Ok(OpportunitySearchDryRun {
        method: "GET",
        surface: "services",
        path: opportunities_search_endpoint(&location_id, &options),
        context,
        location_id,
        limit: options.limit,
        opportunity_status: options.status,
        auth_class: "pit",
        network: false,
    })
}

pub fn get_opportunity_dry_run(
    paths: &crate::ConfigPaths,
    profile_name: Option<&str>,
    location_override: Option<&str>,
    opportunity_id: &str,
) -> Result<OpportunityGetDryRun> {
    validate_opportunity_id(opportunity_id)?;
    let context = resolve_context_for_dry_run(paths, profile_name, location_override, None)?;
    let location_id = context.require_location_id()?.to_owned();

    Ok(OpportunityGetDryRun {
        method: "GET",
        surface: "services",
        path: opportunity_endpoint(opportunity_id),
        context,
        location_id,
        opportunity_id: opportunity_id.to_owned(),
        auth_class: "pit",
        network: false,
    })
}

pub fn create_opportunity_dry_run(
    paths: &crate::ConfigPaths,
    profile_name: Option<&str>,
    location_override: Option<&str>,
    options: OpportunityCreateOptions,
) -> Result<OpportunityWriteDryRun> {
    validate_create_options(&options)?;
    let context = resolve_context_for_dry_run(paths, profile_name, location_override, None)?;
    let location_id = context.require_location_id()?.to_owned();
    let endpoint = opportunity_create_endpoint().to_owned();
    let request_body_json = opportunity_create_body(&location_id, &options)?;
    let request_hash = stable_request_hash(&json!({
        "method": "POST",
        "path": endpoint,
        "body": request_body_json,
    }))?;
    let preflight = OpportunityDuplicatePreflight {
        status: "planned".to_owned(),
        checked: false,
        duplicate_count: 0,
        duplicate_opportunity_ids: Vec::new(),
    };
    let audit = write_opportunity_dry_run_audit(
        paths,
        &context,
        "opportunities.create",
        "opportunity create dry-run; no network mutation performed",
        &endpoint,
        None,
        &request_body_json,
        &request_hash,
        &preflight,
    )?;

    Ok(OpportunityWriteDryRun {
        method: "POST",
        surface: "services",
        path: endpoint,
        context,
        location_id,
        opportunity_id: None,
        request_body_json,
        request_hash,
        idempotency_key: options.idempotency_key,
        preflight,
        audit_entry_id: audit.id,
        auth_class: "pit",
        network: false,
    })
}

pub fn update_opportunity_dry_run(
    paths: &crate::ConfigPaths,
    profile_name: Option<&str>,
    location_override: Option<&str>,
    options: OpportunityUpdateOptions,
) -> Result<OpportunityWriteDryRun> {
    validate_update_options(&options)?;
    let context = resolve_context_for_dry_run(paths, profile_name, location_override, None)?;
    let location_id = context.require_location_id()?.to_owned();
    let endpoint = opportunity_endpoint(&options.opportunity_id);
    let request_body_json = opportunity_update_body(&options)?;
    let request_hash = stable_request_hash(&json!({
        "method": "PUT",
        "path": endpoint,
        "body": request_body_json,
    }))?;
    let preflight = skipped_duplicate_preflight();
    let audit = write_opportunity_dry_run_audit(
        paths,
        &context,
        "opportunities.update",
        "opportunity update dry-run; no network mutation performed",
        &endpoint,
        Some(options.opportunity_id.clone()),
        &request_body_json,
        &request_hash,
        &preflight,
    )?;

    Ok(OpportunityWriteDryRun {
        method: "PUT",
        surface: "services",
        path: endpoint,
        context,
        location_id,
        opportunity_id: Some(options.opportunity_id),
        request_body_json,
        request_hash,
        idempotency_key: options.idempotency_key,
        preflight,
        audit_entry_id: audit.id,
        auth_class: "pit",
        network: false,
    })
}

pub fn create_opportunity(
    paths: &crate::ConfigPaths,
    profile_name: Option<&str>,
    location_override: Option<&str>,
    options: OpportunityCreateOptions,
) -> Result<OpportunityWriteResult> {
    validate_create_options(&options)?;
    let context = resolve_context(paths, profile_name, location_override, None)?;
    ensure_write_allowed(paths, &context.profile)?;
    let location_id = context.require_location_id()?.to_owned();
    let endpoint = opportunity_create_endpoint().to_owned();
    let request_body = opportunity_create_body(&location_id, &options)?;
    let request_hash = stable_request_hash(&json!({
        "method": "POST",
        "path": endpoint,
        "body": request_body,
    }))?;
    let idempotency_key =
        required_idempotency_key(&options.idempotency_key, "real opportunity create")?;

    if let Some(replay) = maybe_replay_opportunity_write(
        paths,
        &context,
        "opportunities.create",
        &endpoint,
        &idempotency_key,
        &request_hash,
        None,
        skipped_duplicate_preflight(),
    )? {
        return Ok(replay);
    }

    let preflight = real_duplicate_preflight(paths, profile_name, &location_id, &options)?;
    record_opportunity_in_progress(
        paths,
        &context,
        "opportunities.create",
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
    let opportunity_id = extract_opportunity_id(response.body_json.as_ref());
    let audit = write_opportunity_result_audit(
        paths,
        &context,
        "opportunities.create",
        &endpoint,
        None,
        &request_body,
        &request_hash,
        &idempotency_key,
        opportunity_id.clone(),
        &preflight,
        response.status,
        response.headers.get("x-request-id").cloned(),
        response.success,
        response
            .body_text
            .clone()
            .or_else(|| response.body_json.as_ref().map(Value::to_string)),
    )?;
    record_opportunity_done(
        paths,
        &context,
        OpportunityDone {
            command: "opportunities.create",
            key: &idempotency_key,
            request_hash: &request_hash,
            opportunity_id: opportunity_id.clone(),
            audit_entry_id: Some(audit.id.clone()),
            success: response.success,
        },
    )?;

    Ok(OpportunityWriteResult {
        profile: context.profile.clone(),
        context,
        location_id,
        endpoint,
        url: response.url,
        status: response.status,
        success: response.success,
        replayed: false,
        opportunity_id,
        idempotency_key: Some(idempotency_key),
        idempotency_state: Some("recorded".to_owned()),
        request_hash,
        preflight,
        audit_entry_id: audit.id,
        body_json: response.body_json,
        body_text: response.body_text,
    })
}

pub fn update_opportunity(
    paths: &crate::ConfigPaths,
    profile_name: Option<&str>,
    location_override: Option<&str>,
    options: OpportunityUpdateOptions,
) -> Result<OpportunityWriteResult> {
    validate_update_options(&options)?;
    let context = resolve_context(paths, profile_name, location_override, None)?;
    ensure_write_allowed(paths, &context.profile)?;
    let location_id = context.require_location_id()?.to_owned();
    let endpoint = opportunity_endpoint(&options.opportunity_id);
    let request_body = opportunity_update_body(&options)?;
    let request_hash = stable_request_hash(&json!({
        "method": "PUT",
        "path": endpoint,
        "body": request_body,
    }))?;
    let idempotency_key =
        required_idempotency_key(&options.idempotency_key, "real opportunity update")?;
    let preflight = skipped_duplicate_preflight();

    if let Some(replay) = maybe_replay_opportunity_write(
        paths,
        &context,
        "opportunities.update",
        &endpoint,
        &idempotency_key,
        &request_hash,
        Some(options.opportunity_id.clone()),
        preflight.clone(),
    )? {
        return Ok(replay);
    }

    record_opportunity_in_progress(
        paths,
        &context,
        "opportunities.update",
        &idempotency_key,
        &request_hash,
        Some(options.opportunity_id.clone()),
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
    let opportunity_id =
        extract_opportunity_id(response.body_json.as_ref()).or(Some(options.opportunity_id));
    let audit = write_opportunity_result_audit(
        paths,
        &context,
        "opportunities.update",
        &endpoint,
        opportunity_id.clone(),
        &request_body,
        &request_hash,
        &idempotency_key,
        opportunity_id.clone(),
        &preflight,
        response.status,
        response.headers.get("x-request-id").cloned(),
        response.success,
        response
            .body_text
            .clone()
            .or_else(|| response.body_json.as_ref().map(Value::to_string)),
    )?;
    record_opportunity_done(
        paths,
        &context,
        OpportunityDone {
            command: "opportunities.update",
            key: &idempotency_key,
            request_hash: &request_hash,
            opportunity_id: opportunity_id.clone(),
            audit_entry_id: Some(audit.id.clone()),
            success: response.success,
        },
    )?;

    Ok(OpportunityWriteResult {
        profile: context.profile.clone(),
        context,
        location_id,
        endpoint,
        url: response.url,
        status: response.status,
        success: response.success,
        replayed: false,
        opportunity_id,
        idempotency_key: Some(idempotency_key),
        idempotency_state: Some("recorded".to_owned()),
        request_hash,
        preflight,
        audit_entry_id: audit.id,
        body_json: response.body_json,
        body_text: response.body_text,
    })
}

fn opportunity_create_endpoint() -> &'static str {
    "/opportunities/"
}

fn opportunity_create_body(location_id: &str, options: &OpportunityCreateOptions) -> Result<Value> {
    let mut body = Map::new();
    body.insert(
        "locationId".to_owned(),
        Value::String(location_id.to_owned()),
    );
    body.insert(
        "name".to_owned(),
        Value::String(options.name.trim().to_owned()),
    );
    body.insert(
        "pipelineId".to_owned(),
        Value::String(options.pipeline_id.trim().to_owned()),
    );
    body.insert(
        "contactId".to_owned(),
        Value::String(options.contact_id.trim().to_owned()),
    );
    body.insert(
        "status".to_owned(),
        Value::String(options.status.as_str().to_owned()),
    );
    insert_optional_string(
        &mut body,
        "pipelineStageId",
        options.pipeline_stage_id.as_deref(),
    );
    insert_optional_string(&mut body, "assignedTo", options.assigned_to.as_deref());
    insert_optional_number(&mut body, "monetaryValue", options.monetary_value)?;
    Ok(Value::Object(body))
}

fn opportunity_update_body(options: &OpportunityUpdateOptions) -> Result<Value> {
    let mut body = Map::new();
    insert_optional_string(&mut body, "name", options.name.as_deref());
    insert_optional_string(&mut body, "pipelineId", options.pipeline_id.as_deref());
    insert_optional_string(
        &mut body,
        "pipelineStageId",
        options.pipeline_stage_id.as_deref(),
    );
    if let Some(status) = options.status {
        body.insert(
            "status".to_owned(),
            Value::String(status.as_str().to_owned()),
        );
    }
    insert_optional_string(&mut body, "assignedTo", options.assigned_to.as_deref());
    insert_optional_number(&mut body, "monetaryValue", options.monetary_value)?;
    Ok(Value::Object(body))
}

fn insert_optional_string(body: &mut Map<String, Value>, key: &str, value: Option<&str>) {
    if let Some(value) = value.and_then(|value| trimmed_optional(Some(value.to_owned()))) {
        body.insert(key.to_owned(), Value::String(value));
    }
}

fn insert_optional_number(
    body: &mut Map<String, Value>,
    key: &str,
    value: Option<f64>,
) -> Result<()> {
    if let Some(value) = value {
        let number = serde_json::Number::from_f64(value).ok_or_else(|| GhlError::Validation {
            message: format!("{key} must be a finite number"),
        })?;
        body.insert(key.to_owned(), Value::Number(number));
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

#[allow(clippy::too_many_arguments)]
fn write_opportunity_dry_run_audit(
    paths: &crate::ConfigPaths,
    context: &ResolvedContext,
    command: &str,
    message: &str,
    endpoint: &str,
    opportunity_id: Option<String>,
    request_body_json: &Value,
    request_hash: &str,
    preflight: &OpportunityDuplicatePreflight,
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
                resource_type: "opportunity".to_owned(),
                id: opportunity_id.clone(),
            }),
            request_summary: json!({
                "endpoint": endpoint,
                "request_body": request_body_json,
                "request_hash": request_hash,
                "preflight": preflight,
            }),
            upstream: None,
            result: AuditResultSummary {
                status: "dry_run".to_owned(),
                resource_id: opportunity_id,
                message: Some(message.to_owned()),
            },
            error: None,
        },
    )
}

#[allow(clippy::too_many_arguments)]
fn write_opportunity_result_audit(
    paths: &crate::ConfigPaths,
    context: &ResolvedContext,
    command: &str,
    endpoint: &str,
    resource_opportunity_id: Option<String>,
    request_body: &Value,
    request_hash: &str,
    idempotency_key: &str,
    result_opportunity_id: Option<String>,
    preflight: &OpportunityDuplicatePreflight,
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
                resource_type: "opportunity".to_owned(),
                id: resource_opportunity_id.or_else(|| result_opportunity_id.clone()),
            }),
            request_summary: json!({
                "endpoint": endpoint,
                "request_body": request_body,
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
                resource_id: result_opportunity_id,
                message: None,
            },
            error: if success { None } else { error_body },
        },
    )
}

#[allow(clippy::too_many_arguments)]
fn maybe_replay_opportunity_write(
    paths: &crate::ConfigPaths,
    context: &ResolvedContext,
    command: &str,
    endpoint: &str,
    idempotency_key: &str,
    request_hash: &str,
    fallback_opportunity_id: Option<String>,
    preflight: OpportunityDuplicatePreflight,
) -> Result<Option<OpportunityWriteResult>> {
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
    let opportunity_id = existing.resource_id.clone().or(fallback_opportunity_id);
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
                resource_type: "opportunity".to_owned(),
                id: opportunity_id.clone(),
            }),
            request_summary: json!({
                "endpoint": endpoint,
                "idempotency_key": idempotency_key,
                "request_hash": request_hash,
            }),
            upstream: None,
            result: AuditResultSummary {
                status: "replayed".to_owned(),
                resource_id: opportunity_id.clone(),
                message: Some(
                    "idempotency key matched a previous request; no network mutation performed"
                        .to_owned(),
                ),
            },
            error: None,
        },
    )?;

    Ok(Some(OpportunityWriteResult {
        profile: context.profile.clone(),
        context: context.clone(),
        location_id,
        endpoint: endpoint.to_owned(),
        url: String::new(),
        status: 200,
        success: existing.status == IdempotencyStatus::Succeeded,
        replayed: true,
        opportunity_id,
        idempotency_key: Some(idempotency_key.to_owned()),
        idempotency_state: Some("replay".to_owned()),
        request_hash: request_hash.to_owned(),
        preflight,
        audit_entry_id: audit.id,
        body_json: None,
        body_text: None,
    }))
}

fn record_opportunity_in_progress(
    paths: &crate::ConfigPaths,
    context: &ResolvedContext,
    command: &str,
    key: &str,
    request_hash: &str,
    opportunity_id: Option<String>,
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
            resource_id: opportunity_id,
            audit_entry_id: None,
        },
    )?;
    Ok(())
}

struct OpportunityDone<'a> {
    command: &'a str,
    key: &'a str,
    request_hash: &'a str,
    opportunity_id: Option<String>,
    audit_entry_id: Option<String>,
    success: bool,
}

fn record_opportunity_done(
    paths: &crate::ConfigPaths,
    context: &ResolvedContext,
    done: OpportunityDone<'_>,
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
            resource_id: done.opportunity_id,
            audit_entry_id: done.audit_entry_id,
        },
    )?;
    Ok(())
}

fn skipped_duplicate_preflight() -> OpportunityDuplicatePreflight {
    OpportunityDuplicatePreflight {
        status: "skipped".to_owned(),
        checked: false,
        duplicate_count: 0,
        duplicate_opportunity_ids: Vec::new(),
    }
}

fn real_duplicate_preflight(
    paths: &crate::ConfigPaths,
    profile_name: Option<&str>,
    location_id: &str,
    options: &OpportunityCreateOptions,
) -> Result<OpportunityDuplicatePreflight> {
    let search_options = OpportunitySearchOptions {
        query: Some(options.name.clone()),
        pipeline_id: Some(options.pipeline_id.clone()),
        pipeline_stage_id: options.pipeline_stage_id.clone(),
        contact_id: Some(options.contact_id.clone()),
        status: Some(options.status),
        assigned_to: None,
        limit: 5,
        page: None,
        start_after_id: None,
        start_after: None,
    };
    let response = raw_get(
        paths,
        profile_name,
        RawGetRequest {
            surface: Surface::Services,
            path: opportunities_search_endpoint(location_id, &search_options),
            auth_class: AuthClass::Pit,
            include_body: true,
        },
    )?;
    let duplicate_ids = matching_duplicate_opportunity_ids(
        response.body_json.as_ref(),
        &options.name,
        &options.pipeline_id,
        &options.contact_id,
    );
    if !duplicate_ids.is_empty() {
        return Err(GhlError::Validation {
            message: format!(
                "opportunity create duplicate preflight found existing opportunity ids: {}",
                duplicate_ids.join(",")
            ),
        });
    }

    Ok(OpportunityDuplicatePreflight {
        status: "passed".to_owned(),
        checked: true,
        duplicate_count: 0,
        duplicate_opportunity_ids: Vec::new(),
    })
}

fn matching_duplicate_opportunity_ids(
    body: Option<&Value>,
    name: &str,
    pipeline_id: &str,
    contact_id: &str,
) -> Vec<String> {
    let Some(body) = body else {
        return Vec::new();
    };
    let opportunities = body
        .get("opportunities")
        .and_then(Value::as_array)
        .map(Vec::as_slice)
        .unwrap_or(&[]);
    opportunities
        .iter()
        .filter(|opportunity| {
            value_eq(opportunity.get("name"), name)
                && value_eq(
                    opportunity
                        .get("pipelineId")
                        .or_else(|| opportunity.get("pipeline_id")),
                    pipeline_id,
                )
                && value_eq(
                    opportunity
                        .get("contactId")
                        .or_else(|| opportunity.get("contact_id")),
                    contact_id,
                )
        })
        .filter_map(|opportunity| {
            opportunity
                .get("id")
                .or_else(|| opportunity.get("opportunityId"))
                .and_then(Value::as_str)
                .map(str::to_owned)
        })
        .collect()
}

fn value_eq(value: Option<&Value>, expected: &str) -> bool {
    value
        .and_then(Value::as_str)
        .is_some_and(|value| value == expected)
}

fn extract_opportunity_id(body: Option<&Value>) -> Option<String> {
    let body = body?;
    body.get("opportunity")
        .and_then(|opportunity| {
            opportunity
                .get("id")
                .or_else(|| opportunity.get("opportunityId"))
                .and_then(Value::as_str)
        })
        .or_else(|| body.get("id").and_then(Value::as_str))
        .or_else(|| body.get("opportunityId").and_then(Value::as_str))
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
            message: "profile policy blocks opportunity writes; enable allow_destructive for this profile before real opportunity mutations".to_owned(),
        });
    }
    Ok(())
}

fn validate_create_options(options: &OpportunityCreateOptions) -> Result<()> {
    validate_optional_text(Some(&options.name), "opportunity name")?;
    validate_path_segment(&options.pipeline_id, "pipeline id")?;
    validate_path_segment(&options.contact_id, "contact id")?;
    validate_optional_path_segment(options.pipeline_stage_id.as_deref(), "pipeline stage id")?;
    validate_optional_path_segment(options.assigned_to.as_deref(), "assigned-to user id")?;
    validate_status_for_write(options.status)?;
    validate_optional_money(options.monetary_value)?;
    validate_optional_text(
        options.idempotency_key.as_deref(),
        "opportunity idempotency key",
    )?;
    Ok(())
}

fn validate_update_options(options: &OpportunityUpdateOptions) -> Result<()> {
    validate_opportunity_id(&options.opportunity_id)?;
    validate_optional_text(options.name.as_deref(), "opportunity name")?;
    validate_optional_path_segment(options.pipeline_id.as_deref(), "pipeline id")?;
    validate_optional_path_segment(options.pipeline_stage_id.as_deref(), "pipeline stage id")?;
    validate_optional_path_segment(options.assigned_to.as_deref(), "assigned-to user id")?;
    if let Some(status) = options.status {
        validate_status_for_write(status)?;
    }
    validate_optional_money(options.monetary_value)?;
    validate_optional_text(
        options.idempotency_key.as_deref(),
        "opportunity idempotency key",
    )?;
    let has_any_field = options
        .name
        .as_deref()
        .is_some_and(|value| !value.trim().is_empty())
        || options
            .pipeline_id
            .as_deref()
            .is_some_and(|value| !value.trim().is_empty())
        || options
            .pipeline_stage_id
            .as_deref()
            .is_some_and(|value| !value.trim().is_empty())
        || options.status.is_some()
        || options.monetary_value.is_some()
        || options
            .assigned_to
            .as_deref()
            .is_some_and(|value| !value.trim().is_empty());
    if !has_any_field {
        return Err(GhlError::Validation {
            message: "opportunity update requires at least one field".to_owned(),
        });
    }
    Ok(())
}

fn validate_status_for_write(status: OpportunityStatus) -> Result<()> {
    if status == OpportunityStatus::All {
        return Err(GhlError::Validation {
            message: "opportunity writes do not accept status `all`".to_owned(),
        });
    }
    Ok(())
}

fn validate_optional_money(value: Option<f64>) -> Result<()> {
    if let Some(value) = value
        && !value.is_finite()
    {
        return Err(GhlError::Validation {
            message: "opportunity monetary value must be finite".to_owned(),
        });
    }
    Ok(())
}

fn validate_optional_path_segment(value: Option<&str>, label: &str) -> Result<()> {
    if let Some(value) = value {
        validate_path_segment(value, label)?;
    }
    Ok(())
}

fn opportunities_search_endpoint(location_id: &str, options: &OpportunitySearchOptions) -> String {
    let mut serializer = Serializer::new(String::new());
    serializer.append_pair("location_id", location_id);
    serializer.append_pair("limit", &options.limit.to_string());
    if let Some(query) = trimmed_optional(options.query.clone()) {
        serializer.append_pair("q", &query);
    }
    if let Some(pipeline_id) = trimmed_optional(options.pipeline_id.clone()) {
        serializer.append_pair("pipeline_id", &pipeline_id);
    }
    if let Some(pipeline_stage_id) = trimmed_optional(options.pipeline_stage_id.clone()) {
        serializer.append_pair("pipeline_stage_id", &pipeline_stage_id);
    }
    if let Some(contact_id) = trimmed_optional(options.contact_id.clone()) {
        serializer.append_pair("contact_id", &contact_id);
    }
    if let Some(status) = options.status {
        serializer.append_pair("status", status.as_str());
    }
    if let Some(assigned_to) = trimmed_optional(options.assigned_to.clone()) {
        serializer.append_pair("assigned_to", &assigned_to);
    }
    if let Some(page) = options.page {
        serializer.append_pair("page", &page.to_string());
    }
    if let Some(start_after_id) = trimmed_optional(options.start_after_id.clone()) {
        serializer.append_pair("startAfterId", &start_after_id);
    }
    if let Some(start_after) = options.start_after {
        serializer.append_pair("startAfter", &start_after.to_string());
    }

    format!("/opportunities/search?{}", serializer.finish())
}

fn opportunity_endpoint(opportunity_id: &str) -> String {
    format!("/opportunities/{opportunity_id}")
}

fn validate_search_options(options: &OpportunitySearchOptions) -> Result<()> {
    if options.limit == 0 || options.limit > 100 {
        return Err(GhlError::Validation {
            message: "opportunity search limit must be between 1 and 100".to_owned(),
        });
    }
    validate_optional_text(options.query.as_deref(), "opportunity query")?;
    validate_optional_text(options.pipeline_id.as_deref(), "pipeline id")?;
    validate_optional_text(options.pipeline_stage_id.as_deref(), "pipeline stage id")?;
    validate_optional_text(options.contact_id.as_deref(), "contact id")?;
    validate_optional_text(options.assigned_to.as_deref(), "assigned-to user id")?;
    validate_optional_text(options.start_after_id.as_deref(), "start-after id")?;

    Ok(())
}

fn validate_opportunity_id(opportunity_id: &str) -> Result<()> {
    validate_path_segment(opportunity_id, "opportunity id")
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
    fn opportunities_search_uses_location_context_and_filters() {
        let server = MockServer::start();
        let mock = server.mock(|when, then| {
            when.method(GET)
                .path("/opportunities/search")
                .query_param("location_id", "loc_123")
                .query_param("limit", "20")
                .query_param("q", "Roof")
                .query_param("pipeline_id", "pipe_123")
                .query_param("pipeline_stage_id", "stage_123")
                .query_param("contact_id", "contact_123")
                .query_param("status", "open")
                .header("authorization", "Bearer pit-secret");
            then.status(200).json_body(json!({
                "opportunities": [
                    {
                        "id": "opp_123",
                        "name": "Roof Repair",
                        "pipelineId": "pipe_123",
                        "pipelineStageId": "stage_123",
                        "status": "open",
                        "notes": ["private note"]
                    }
                ],
                "meta": { "total": 1 }
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
        point_services_base_url(&paths, &server);

        let result = search_opportunities(
            &paths,
            Some("default"),
            None,
            OpportunitySearchOptions {
                query: Some("Roof".to_owned()),
                pipeline_id: Some("pipe_123".to_owned()),
                pipeline_stage_id: Some("stage_123".to_owned()),
                contact_id: Some("contact_123".to_owned()),
                status: Some(OpportunityStatus::Open),
                assigned_to: None,
                limit: 20,
                page: None,
                start_after_id: None,
                start_after: None,
            },
        )
        .expect("opportunities");

        mock.assert();
        assert_eq!(result.location_id, "loc_123");
        assert_eq!(result.opportunity_status, Some(OpportunityStatus::Open));
        assert_eq!(
            result.body_json.unwrap()["opportunities"][0]["notes"],
            "[REDACTED]"
        );
    }

    #[test]
    fn opportunity_get_requires_location_context() {
        let server = MockServer::start();
        let mock = server.mock(|when, then| {
            when.method(GET)
                .path("/opportunities/opp_123")
                .header("authorization", "Bearer pit-secret");
            then.status(200).json_body(json!({
                "opportunity": {
                    "id": "opp_123",
                    "name": "Roof Repair",
                    "notes": ["private note"]
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
        point_services_base_url(&paths, &server);

        let result =
            get_opportunity(&paths, Some("default"), None, "opp_123").expect("opportunity");

        mock.assert();
        assert_eq!(result.location_id, "loc_123");
        assert_eq!(
            result.body_json.unwrap()["opportunity"]["notes"],
            "[REDACTED]"
        );
    }

    #[test]
    fn opportunities_search_dry_run_uses_location_override_without_profile() {
        let temp = tempfile::tempdir().expect("tempdir");
        let paths = resolve_paths(Some(temp.path()));

        let result = opportunities_search_dry_run(
            &paths,
            Some("missing"),
            Some("loc_override"),
            OpportunitySearchOptions {
                query: None,
                pipeline_id: Some("pipe_123".to_owned()),
                pipeline_stage_id: None,
                contact_id: None,
                status: Some(OpportunityStatus::Won),
                assigned_to: None,
                limit: 10,
                page: None,
                start_after_id: Some("opp_099".to_owned()),
                start_after: None,
            },
        )
        .expect("dry run");

        assert_eq!(result.location_id, "loc_override");
        assert_eq!(
            result.path,
            "/opportunities/search?location_id=loc_override&limit=10&pipeline_id=pipe_123&status=won&startAfterId=opp_099"
        );
    }

    #[test]
    fn opportunity_get_rejects_path_injection() {
        let temp = tempfile::tempdir().expect("tempdir");
        let paths = resolve_paths(Some(temp.path()));

        let error = get_opportunity_dry_run(&paths, None, Some("loc_123"), "../opp_123")
            .expect_err("invalid id");

        assert_eq!(error.code(), "validation_error");
    }

    #[test]
    fn opportunity_create_dry_run_writes_audit() {
        let temp = tempfile::tempdir().expect("tempdir");
        let paths = resolve_paths(Some(temp.path()));

        let result =
            create_opportunity_dry_run(&paths, None, Some("loc_123"), opportunity_create_options())
                .expect("dry run");

        assert_eq!(result.method, "POST");
        assert_eq!(result.path, "/opportunities/");
        assert_eq!(result.request_body_json["locationId"], "loc_123");
        assert_eq!(result.request_body_json["name"], "Roof Repair");
        assert_eq!(result.request_body_json["pipelineId"], "pipe_123");
        assert_eq!(result.request_body_json["contactId"], "contact_123");
        assert_eq!(result.request_body_json["pipelineStageId"], "stage_123");
        assert_eq!(result.request_body_json["status"], "open");
        assert_eq!(result.request_body_json["monetaryValue"], 50000.0);
        assert_eq!(result.preflight.status, "planned");
        let audit =
            std::fs::read_to_string(paths.audit_dir.join("audit.jsonl")).expect("audit journal");
        assert!(audit.contains("opportunities.create"));
        assert!(audit.contains("Roof Repair"));
    }

    #[test]
    fn opportunity_create_posts_after_policy_idempotency_and_duplicate_preflight() {
        let server = MockServer::start();
        let duplicate = server.mock(|when, then| {
            when.method(GET)
                .path("/opportunities/search")
                .query_param("location_id", "loc_123")
                .query_param("limit", "5")
                .query_param("q", "Roof Repair")
                .query_param("pipeline_id", "pipe_123")
                .query_param("pipeline_stage_id", "stage_123")
                .query_param("contact_id", "contact_123")
                .query_param("status", "open")
                .header("authorization", "Bearer pit-secret");
            then.status(200)
                .header("content-type", "application/json")
                .json_body(json!({ "opportunities": [], "total": 0 }));
        });
        let create = server.mock(|when, then| {
            when.method(POST)
                .path("/opportunities/")
                .header("authorization", "Bearer pit-secret")
                .json_body(json!({
                    "locationId": "loc_123",
                    "name": "Roof Repair",
                    "pipelineId": "pipe_123",
                    "contactId": "contact_123",
                    "status": "open",
                    "pipelineStageId": "stage_123",
                    "assignedTo": "user_123",
                    "monetaryValue": 50000.0
                }));
            then.status(201)
                .header("content-type", "application/json")
                .json_body(json!({
                    "opportunity": {
                        "id": "opp_123",
                        "name": "Roof Repair"
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
        point_services_base_url_with_destructive(&paths, &server);

        let result =
            create_opportunity(&paths, Some("default"), None, opportunity_create_options())
                .expect("create opportunity");

        duplicate.assert();
        create.assert();
        assert!(result.success);
        assert_eq!(result.opportunity_id, Some("opp_123".to_owned()));
        assert_eq!(result.preflight.status, "passed");
        let idempotency = crate::list_idempotency_records(&paths).expect("idempotency");
        assert_eq!(idempotency.count, 1);
        assert_eq!(
            idempotency.records[0].resource_id,
            Some("opp_123".to_owned())
        );
    }

    #[test]
    fn opportunity_create_duplicate_preflight_blocks_existing_opportunity() {
        let server = MockServer::start();
        let duplicate = server.mock(|when, then| {
            when.method(GET).path("/opportunities/search");
            then.status(200)
                .header("content-type", "application/json")
                .json_body(json!({
                    "opportunities": [
                        {
                            "id": "opp_existing",
                            "name": "Roof Repair",
                            "pipelineId": "pipe_123",
                            "contactId": "contact_123"
                        }
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
        point_services_base_url_with_destructive(&paths, &server);

        let error = create_opportunity(&paths, Some("default"), None, opportunity_create_options())
            .expect_err("duplicate");

        duplicate.assert();
        assert_eq!(error.code(), "validation_error");
        assert!(error.to_string().contains("opp_existing"));
    }

    #[test]
    fn opportunity_update_puts_after_policy_and_idempotency() {
        let server = MockServer::start();
        let update = server.mock(|when, then| {
            when.method(PUT)
                .path("/opportunities/opp_123")
                .header("authorization", "Bearer pit-secret")
                .json_body(json!({
                    "pipelineStageId": "stage_456",
                    "status": "won",
                    "monetaryValue": 60000.0
                }));
            then.status(200)
                .header("content-type", "application/json")
                .json_body(json!({
                    "opportunity": {
                        "id": "opp_123",
                        "status": "won"
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
        point_services_base_url_with_destructive(&paths, &server);

        let result = update_opportunity(
            &paths,
            Some("default"),
            None,
            OpportunityUpdateOptions {
                opportunity_id: "opp_123".to_owned(),
                name: None,
                pipeline_id: None,
                pipeline_stage_id: Some("stage_456".to_owned()),
                status: Some(OpportunityStatus::Won),
                monetary_value: Some(60000.0),
                assigned_to: None,
                idempotency_key: Some("update-opportunity-1".to_owned()),
            },
        )
        .expect("update opportunity");

        update.assert();
        assert!(result.success);
        assert_eq!(result.opportunity_id, Some("opp_123".to_owned()));
        assert_eq!(result.preflight.status, "skipped");
    }

    fn point_services_base_url(paths: &crate::ConfigPaths, server: &MockServer) {
        let mut profiles = crate::profiles::load_profiles(paths).expect("profiles");
        profiles
            .profiles
            .get_mut("default")
            .expect("profile")
            .base_urls
            .services = server.base_url();
        crate::profiles::save_profiles(paths, &profiles).expect("save");
    }

    fn point_services_base_url_with_destructive(paths: &crate::ConfigPaths, server: &MockServer) {
        let mut profiles = crate::profiles::load_profiles(paths).expect("profiles");
        let profile = profiles.profiles.get_mut("default").expect("profile");
        profile.base_urls.services = server.base_url();
        profile.policy.allow_destructive = true;
        crate::profiles::save_profiles(paths, &profiles).expect("save");
    }

    fn opportunity_create_options() -> OpportunityCreateOptions {
        OpportunityCreateOptions {
            name: "Roof Repair".to_owned(),
            pipeline_id: "pipe_123".to_owned(),
            pipeline_stage_id: Some("stage_123".to_owned()),
            contact_id: "contact_123".to_owned(),
            status: OpportunityStatus::Open,
            monetary_value: Some(50000.0),
            assigned_to: Some("user_123".to_owned()),
            idempotency_key: Some("create-opportunity-1".to_owned()),
        }
    }
}
