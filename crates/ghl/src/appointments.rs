use chrono::{DateTime, FixedOffset};
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value, json};

use crate::audit::{
    AuditEntryInput, AuditResource, AuditResultSummary, AuditUpstreamSummary, append_audit_entry,
};
use crate::client::{AuthClass, RawPostJsonRequest, post_json};
use crate::context::{ResolvedContext, resolve_context, resolve_context_for_dry_run};
use crate::errors::{GhlError, Result};
use crate::idempotency::{
    IdempotencyCheckState, IdempotencyPut, IdempotencyStatus, check_idempotency_key,
    record_idempotency_key, stable_request_hash,
};
use crate::profiles::load_profiles;
use crate::redaction::redact_json;
use crate::surfaces::Surface;
use crate::{CalendarFreeSlotsOptions, get_calendar_free_slots};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AppointmentCreateOptions {
    pub calendar_id: String,
    pub contact_id: String,
    pub starts_at: String,
    pub ends_at: String,
    pub title: Option<String>,
    pub appointment_status: AppointmentStatus,
    pub assigned_user_id: Option<String>,
    pub address: Option<String>,
    pub meeting_location_type: Option<String>,
    pub timezone: Option<String>,
    pub ignore_date_range: bool,
    pub to_notify: bool,
    pub idempotency_key: Option<String>,
    pub skip_free_slot_check: bool,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum AppointmentStatus {
    New,
    Confirmed,
}

impl AppointmentStatus {
    pub fn as_api_value(self) -> &'static str {
        match self {
            Self::New => "new",
            Self::Confirmed => "confirmed",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AppointmentCreateDryRun {
    pub method: &'static str,
    pub surface: &'static str,
    pub path: String,
    pub context: ResolvedContext,
    pub location_id: String,
    pub request_body_json: Value,
    pub request_hash: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub idempotency_key: Option<String>,
    pub preflight: AppointmentPreflightSummary,
    pub audit_entry_id: String,
    pub auth_class: &'static str,
    pub network: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AppointmentCreateResult {
    pub profile: String,
    pub context: ResolvedContext,
    pub location_id: String,
    pub endpoint: String,
    pub url: String,
    pub status: u16,
    pub success: bool,
    pub replayed: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub appointment_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub idempotency_key: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub idempotency_state: Option<String>,
    pub request_hash: String,
    pub preflight: AppointmentPreflightSummary,
    pub audit_entry_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub body_json: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub body_text: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AppointmentPreflightSummary {
    pub free_slot_check: PreflightStatus,
    pub duplicate_check: PreflightStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PreflightStatus {
    pub status: String,
    pub checked: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub detail: Option<String>,
}

pub fn create_appointment_dry_run(
    paths: &crate::ConfigPaths,
    profile_name: Option<&str>,
    location_override: Option<&str>,
    options: AppointmentCreateOptions,
) -> Result<AppointmentCreateDryRun> {
    let parsed = validate_create_options(&options)?;
    let context = resolve_context_for_dry_run(paths, profile_name, location_override, None)?;
    let location_id = context.require_location_id()?.to_owned();
    let request_body_json = appointment_create_body(&location_id, &options);
    let request_hash = stable_request_hash(&request_body_json)?;
    let preflight = dry_run_preflight(&options);
    let audit = append_audit_entry(
        paths,
        AuditEntryInput {
            profile: Some(context.profile.clone()),
            company_id: context.company_id.as_ref().map(|value| value.value.clone()),
            location_id: Some(location_id.clone()),
            command: "appointments.create".to_owned(),
            action_class: "sensitive_dry_run".to_owned(),
            dry_run: true,
            policy_flags: vec![
                "allow_destructive".to_owned(),
                "confirmation_required".to_owned(),
            ],
            resource: Some(AuditResource {
                resource_type: "appointment".to_owned(),
                id: None,
            }),
            request_summary: json!({
                "endpoint": appointment_create_endpoint(),
                "request_body": request_body_json,
                "starts_at_unix_ms": parsed.starts_at_ms,
                "ends_at_unix_ms": parsed.ends_at_ms,
                "preflight": preflight,
            }),
            upstream: None,
            result: AuditResultSummary {
                status: "dry_run".to_owned(),
                resource_id: None,
                message: Some(
                    "appointment create dry-run; no network mutation performed".to_owned(),
                ),
            },
            error: None,
        },
    )?;

    Ok(AppointmentCreateDryRun {
        method: "POST",
        surface: "services",
        path: appointment_create_endpoint().to_owned(),
        context,
        location_id,
        request_body_json: redact_json(&request_body_json),
        request_hash,
        idempotency_key: options.idempotency_key,
        preflight,
        audit_entry_id: audit.id,
        auth_class: "pit",
        network: false,
    })
}

pub fn create_appointment(
    paths: &crate::ConfigPaths,
    profile_name: Option<&str>,
    location_override: Option<&str>,
    options: AppointmentCreateOptions,
) -> Result<AppointmentCreateResult> {
    let parsed = validate_create_options(&options)?;
    let context = resolve_context(paths, profile_name, location_override, None)?;
    ensure_write_allowed(paths, &context.profile)?;
    let location_id = context.require_location_id()?.to_owned();
    let request_body = appointment_create_body(&location_id, &options);
    let request_hash = stable_request_hash(&request_body)?;
    let idempotency_key = options
        .idempotency_key
        .clone()
        .ok_or_else(|| GhlError::Validation {
            message: "real appointment create requires --idempotency-key <key>".to_owned(),
        })?;
    let idempotency = check_idempotency_key(
        paths,
        &context.profile,
        Some(&location_id),
        "appointments.create",
        &idempotency_key,
        &request_hash,
    )?;
    if idempotency.state == IdempotencyCheckState::Replay {
        let existing = idempotency.existing.expect("existing replay record");
        let audit = append_audit_entry(
            paths,
            AuditEntryInput {
                profile: Some(context.profile.clone()),
                company_id: context.company_id.as_ref().map(|value| value.value.clone()),
                location_id: Some(location_id.clone()),
                command: "appointments.create".to_owned(),
                action_class: "idempotency_replay".to_owned(),
                dry_run: false,
                policy_flags: vec![
                    "allow_destructive".to_owned(),
                    "confirmation_required".to_owned(),
                ],
                resource: Some(AuditResource {
                    resource_type: "appointment".to_owned(),
                    id: existing.resource_id.clone(),
                }),
                request_summary: json!({
                    "endpoint": appointment_create_endpoint(),
                    "idempotency_key": idempotency_key,
                    "request_hash": request_hash,
                }),
                upstream: None,
                result: AuditResultSummary {
                    status: "replayed".to_owned(),
                    resource_id: existing.resource_id.clone(),
                    message: Some(
                        "idempotency key matched a previous request; no network mutation performed"
                            .to_owned(),
                    ),
                },
                error: None,
            },
        )?;
        return Ok(AppointmentCreateResult {
            profile: context.profile.clone(),
            context,
            location_id,
            endpoint: appointment_create_endpoint().to_owned(),
            url: String::new(),
            status: 200,
            success: existing.status == IdempotencyStatus::Succeeded,
            replayed: true,
            appointment_id: existing.resource_id,
            idempotency_key: Some(idempotency_key),
            idempotency_state: Some("replay".to_owned()),
            request_hash,
            preflight: replay_preflight(),
            audit_entry_id: audit.id,
            body_json: None,
            body_text: None,
        });
    }

    record_idempotency_key(
        paths,
        IdempotencyPut {
            key: idempotency_key.clone(),
            profile: context.profile.clone(),
            location_id: Some(location_id.clone()),
            command: "appointments.create".to_owned(),
            request_hash: request_hash.clone(),
            status: IdempotencyStatus::InProgress,
            resource_id: None,
            audit_entry_id: None,
        },
    )?;

    let preflight = real_preflight(paths, profile_name, location_override, &options, &parsed)?;
    let response = post_json(
        paths,
        profile_name,
        RawPostJsonRequest {
            surface: Surface::Services,
            path: appointment_create_endpoint().to_owned(),
            auth_class: AuthClass::Pit,
            body: request_body.clone(),
            include_body: true,
        },
    )?;
    let appointment_id = extract_appointment_id(response.body_json.as_ref());
    let result_status = if response.success {
        "success"
    } else {
        "failed"
    }
    .to_owned();
    let idempotency_status = if response.success {
        IdempotencyStatus::Succeeded
    } else {
        IdempotencyStatus::Failed
    };
    let audit = append_audit_entry(
        paths,
        AuditEntryInput {
            profile: Some(context.profile.clone()),
            company_id: context.company_id.as_ref().map(|value| value.value.clone()),
            location_id: Some(location_id.clone()),
            command: "appointments.create".to_owned(),
            action_class: "write".to_owned(),
            dry_run: false,
            policy_flags: vec![
                "allow_destructive".to_owned(),
                "confirmation_required".to_owned(),
            ],
            resource: Some(AuditResource {
                resource_type: "appointment".to_owned(),
                id: appointment_id.clone(),
            }),
            request_summary: json!({
                "endpoint": appointment_create_endpoint(),
                "request_body": request_body,
                "request_hash": request_hash,
                "idempotency_key": idempotency_key,
                "preflight": preflight,
            }),
            upstream: Some(AuditUpstreamSummary {
                request_id: response.headers.get("x-request-id").cloned(),
                status_code: Some(response.status),
                endpoint_key: Some("appointments.create".to_owned()),
            }),
            result: AuditResultSummary {
                status: result_status,
                resource_id: appointment_id.clone(),
                message: None,
            },
            error: if response.success {
                None
            } else {
                response
                    .body_text
                    .clone()
                    .or_else(|| response.body_json.as_ref().map(Value::to_string))
            },
        },
    )?;
    record_idempotency_key(
        paths,
        IdempotencyPut {
            key: idempotency_key.clone(),
            profile: context.profile.clone(),
            location_id: Some(location_id.clone()),
            command: "appointments.create".to_owned(),
            request_hash: request_hash.clone(),
            status: idempotency_status,
            resource_id: appointment_id.clone(),
            audit_entry_id: Some(audit.id.clone()),
        },
    )?;

    Ok(AppointmentCreateResult {
        profile: context.profile.clone(),
        context,
        location_id,
        endpoint: appointment_create_endpoint().to_owned(),
        url: response.url,
        status: response.status,
        success: response.success,
        replayed: false,
        appointment_id,
        idempotency_key: Some(idempotency_key),
        idempotency_state: Some("recorded".to_owned()),
        request_hash,
        preflight,
        audit_entry_id: audit.id,
        body_json: response.body_json,
        body_text: response.body_text,
    })
}

fn ensure_write_allowed(paths: &crate::ConfigPaths, profile_name: &str) -> Result<()> {
    let profiles = load_profiles(paths)?;
    let profile = profiles.get_required(profile_name)?;
    if !profile.policy.allow_destructive {
        return Err(GhlError::PolicyDenied {
            message: "profile policy blocks appointment writes; enable allow_destructive for this profile before real create".to_owned(),
        });
    }
    Ok(())
}

fn appointment_create_endpoint() -> &'static str {
    "/calendars/events/appointments"
}

fn appointment_create_body(location_id: &str, options: &AppointmentCreateOptions) -> Value {
    let mut body = Map::new();
    body.insert(
        "locationId".to_owned(),
        Value::String(location_id.to_owned()),
    );
    body.insert(
        "calendarId".to_owned(),
        Value::String(options.calendar_id.trim().to_owned()),
    );
    body.insert(
        "contactId".to_owned(),
        Value::String(options.contact_id.trim().to_owned()),
    );
    body.insert(
        "startTime".to_owned(),
        Value::String(options.starts_at.trim().to_owned()),
    );
    body.insert(
        "endTime".to_owned(),
        Value::String(options.ends_at.trim().to_owned()),
    );
    body.insert(
        "appointmentStatus".to_owned(),
        Value::String(options.appointment_status.as_api_value().to_owned()),
    );
    body.insert("toNotify".to_owned(), Value::Bool(options.to_notify));
    body.insert(
        "ignoreDateRange".to_owned(),
        Value::Bool(options.ignore_date_range),
    );
    insert_optional(&mut body, "title", options.title.as_deref());
    insert_optional(
        &mut body,
        "assignedUserId",
        options.assigned_user_id.as_deref(),
    );
    insert_optional(&mut body, "address", options.address.as_deref());
    insert_optional(
        &mut body,
        "meetingLocationType",
        options.meeting_location_type.as_deref(),
    );
    insert_optional(&mut body, "timezone", options.timezone.as_deref());
    Value::Object(body)
}

fn insert_optional(body: &mut Map<String, Value>, key: &str, value: Option<&str>) {
    if let Some(value) = value.map(str::trim).filter(|value| !value.is_empty()) {
        body.insert(key.to_owned(), Value::String(value.to_owned()));
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ParsedAppointmentRange {
    starts_at: DateTime<FixedOffset>,
    starts_at_ms: u64,
    ends_at_ms: u64,
}

fn validate_create_options(options: &AppointmentCreateOptions) -> Result<ParsedAppointmentRange> {
    validate_path_segment(&options.calendar_id, "calendar id")?;
    validate_path_segment(&options.contact_id, "contact id")?;
    validate_optional_text(options.title.as_deref(), "appointment title")?;
    validate_optional_text(options.assigned_user_id.as_deref(), "assigned user id")?;
    validate_optional_text(options.address.as_deref(), "appointment address")?;
    validate_optional_text(
        options.meeting_location_type.as_deref(),
        "meeting location type",
    )?;
    validate_optional_text(options.timezone.as_deref(), "appointment timezone")?;
    validate_optional_text(options.idempotency_key.as_deref(), "idempotency key")?;
    let starts_at = parse_datetime(&options.starts_at, "--starts-at")?;
    let ends_at = parse_datetime(&options.ends_at, "--ends-at")?;
    if ends_at <= starts_at {
        return Err(GhlError::Validation {
            message: "appointment --ends-at must be after --starts-at".to_owned(),
        });
    }
    Ok(ParsedAppointmentRange {
        starts_at,
        starts_at_ms: millis_to_u64(starts_at.timestamp_millis(), "appointment start")?,
        ends_at_ms: millis_to_u64(ends_at.timestamp_millis(), "appointment end")?,
    })
}

fn parse_datetime(value: &str, label: &str) -> Result<DateTime<FixedOffset>> {
    let value = value.trim();
    if value.is_empty() {
        return Err(GhlError::Validation {
            message: format!("{label} cannot be empty"),
        });
    }
    DateTime::parse_from_rfc3339(value).map_err(|_| GhlError::Validation {
        message: format!("{label} must be an RFC3339 datetime with timezone offset"),
    })
}

fn millis_to_u64(value: i64, label: &str) -> Result<u64> {
    u64::try_from(value).map_err(|_| GhlError::Validation {
        message: format!("{label} must not be before the Unix epoch"),
    })
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

fn dry_run_preflight(options: &AppointmentCreateOptions) -> AppointmentPreflightSummary {
    AppointmentPreflightSummary {
        free_slot_check: if options.skip_free_slot_check {
            PreflightStatus {
                status: "skipped".to_owned(),
                checked: false,
                detail: Some("--skip-free-slot-check was set".to_owned()),
            }
        } else {
            PreflightStatus {
                status: "planned".to_owned(),
                checked: false,
                detail: Some("real create checks calendar free slots before mutation".to_owned()),
            }
        },
        duplicate_check: PreflightStatus {
            status: "planned".to_owned(),
            checked: false,
            detail: Some("contact/time duplicate check is planned; idempotency key is active for real creates".to_owned()),
        },
    }
}

fn replay_preflight() -> AppointmentPreflightSummary {
    AppointmentPreflightSummary {
        free_slot_check: PreflightStatus {
            status: "skipped".to_owned(),
            checked: false,
            detail: Some("idempotency replay avoided a network mutation".to_owned()),
        },
        duplicate_check: PreflightStatus {
            status: "matched_idempotency_key".to_owned(),
            checked: true,
            detail: None,
        },
    }
}

fn real_preflight(
    paths: &crate::ConfigPaths,
    profile_name: Option<&str>,
    location_override: Option<&str>,
    options: &AppointmentCreateOptions,
    parsed: &ParsedAppointmentRange,
) -> Result<AppointmentPreflightSummary> {
    let free_slot_check = if options.skip_free_slot_check {
        PreflightStatus {
            status: "skipped".to_owned(),
            checked: false,
            detail: Some("--skip-free-slot-check was set".to_owned()),
        }
    } else {
        let date = parsed.starts_at.date_naive().format("%Y-%m-%d").to_string();
        let slots = get_calendar_free_slots(
            paths,
            profile_name,
            location_override,
            CalendarFreeSlotsOptions {
                calendar_id: options.calendar_id.clone(),
                date,
                timezone: options.timezone.clone(),
                user_id: options.assigned_user_id.clone(),
                enable_look_busy: false,
            },
        )?;
        let available = slot_is_available(slots.body_json.as_ref(), parsed.starts_at_ms);
        if !available {
            return Err(GhlError::Validation {
                message: "requested appointment start was not present in the free-slot response; pass --skip-free-slot-check only if you have verified availability another way".to_owned(),
            });
        }
        PreflightStatus {
            status: "passed".to_owned(),
            checked: true,
            detail: Some(format!(
                "free-slot response contained {} slots",
                slots.slot_count
            )),
        }
    };

    Ok(AppointmentPreflightSummary {
        free_slot_check,
        duplicate_check: PreflightStatus {
            status: "idempotency_key_checked".to_owned(),
            checked: true,
            detail: Some("contact/time duplicate endpoint is not wired yet".to_owned()),
        },
    })
}

fn slot_is_available(body: Option<&Value>, starts_at_ms: u64) -> bool {
    let Some(Value::Object(days)) = body else {
        return false;
    };
    days.values()
        .filter_map(|day| day.get("slots").and_then(Value::as_array))
        .flatten()
        .any(|slot| slot_matches_start(slot, starts_at_ms))
}

fn slot_matches_start(slot: &Value, starts_at_ms: u64) -> bool {
    if let Some(value) = slot.as_u64() {
        return value == starts_at_ms;
    }
    let Some(value) = slot.as_str() else {
        return false;
    };
    if let Ok(value) = value.parse::<u64>() {
        return value == starts_at_ms;
    }
    DateTime::parse_from_rfc3339(value)
        .map(|datetime| millis_to_u64(datetime.timestamp_millis(), "slot time").ok())
        .ok()
        .flatten()
        == Some(starts_at_ms)
}

fn extract_appointment_id(body: Option<&Value>) -> Option<String> {
    let body = body?;
    body.get("id")
        .or_else(|| body.get("eventId"))
        .or_else(|| body.get("appointmentId"))
        .or_else(|| body.get("event").and_then(|event| event.get("id")))
        .or_else(|| {
            body.get("appointment")
                .and_then(|appointment| appointment.get("id"))
        })
        .and_then(Value::as_str)
        .map(str::to_owned)
}

#[cfg(test)]
mod tests {
    use httpmock::Method::{GET, POST};
    use httpmock::MockServer;
    use serde_json::json;

    use super::*;
    use crate::auth::add_pit;
    use crate::config::resolve_paths;

    #[test]
    fn appointment_create_dry_run_writes_redacted_audit_entry() {
        let temp = tempfile::tempdir().expect("tempdir");
        let paths = resolve_paths(Some(temp.path()));

        let result = create_appointment_dry_run(
            &paths,
            None,
            Some("loc_123"),
            options(Some("key-1"), false),
        )
        .expect("dry run");

        assert_eq!(result.method, "POST");
        assert_eq!(result.path, "/calendars/events/appointments");
        assert_eq!(result.request_body_json["locationId"], "loc_123");
        assert_eq!(result.preflight.free_slot_check.status, "planned");
        assert!(result.audit_entry_id.starts_with("audit-"));
        let audit = std::fs::read_to_string(crate::audit_journal_path(&paths)).expect("audit");
        assert!(audit.contains("appointments.create"));
    }

    #[test]
    fn appointment_create_requires_end_after_start() {
        let temp = tempfile::tempdir().expect("tempdir");
        let paths = resolve_paths(Some(temp.path()));
        let mut invalid = options(None, false);
        invalid.ends_at = invalid.starts_at.clone();

        let error = create_appointment_dry_run(&paths, None, Some("loc_123"), invalid)
            .expect_err("invalid range");

        assert_eq!(error.code(), "validation_error");
    }

    #[test]
    fn appointment_create_posts_after_policy_idempotency_and_free_slot_check() {
        let server = MockServer::start();
        let slots = server.mock(|when, then| {
            when.method(GET)
                .path("/calendars/cal_123/free-slots")
                .query_param("startDate", "1772150400000")
                .query_param("endDate", "1772236800000")
                .query_param("timezone", "America/Denver")
                .query_param("userId", "user_123");
            then.status(200).json_body(json!({
                "2026-02-27": { "slots": ["2026-02-27T16:00:00Z"] }
            }));
        });
        let create = server.mock(|when, then| {
            when.method(POST)
                .path("/calendars/events/appointments")
                .header("authorization", "Bearer pit-secret")
                .json_body(json!({
                    "locationId": "loc_123",
                    "calendarId": "cal_123",
                    "contactId": "contact_123",
                    "startTime": "2026-02-27T09:00:00-07:00",
                    "endTime": "2026-02-27T09:30:00-07:00",
                    "appointmentStatus": "confirmed",
                    "toNotify": false,
                    "ignoreDateRange": false,
                    "title": "Discovery Call",
                    "assignedUserId": "user_123",
                    "meetingLocationType": "phone",
                    "timezone": "America/Denver"
                }));
            then.status(201).json_body(json!({ "id": "appt_123" }));
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

        let result = create_appointment(
            &paths,
            Some("default"),
            None,
            options(Some("create-appt-1"), false),
        )
        .expect("create");

        slots.assert();
        create.assert();
        assert!(result.success);
        assert_eq!(result.appointment_id, Some("appt_123".to_owned()));
        assert_eq!(result.preflight.free_slot_check.status, "passed");
        assert!(result.audit_entry_id.starts_with("audit-"));
        let idempotency = crate::list_idempotency_records(&paths).expect("idempotency");
        assert_eq!(idempotency.count, 1);
        assert_eq!(
            idempotency.records[0].resource_id,
            Some("appt_123".to_owned())
        );
    }

    #[test]
    fn appointment_create_replays_matching_idempotency_key_without_network() {
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
            .policy
            .allow_destructive = true;
        crate::profiles::save_profiles(&paths, &profiles).expect("save");
        let opts = options(Some("create-appt-1"), true);
        let body = appointment_create_body("loc_123", &opts);
        let request_hash = stable_request_hash(&body).expect("hash");
        crate::record_idempotency_key(
            &paths,
            IdempotencyPut {
                key: "create-appt-1".to_owned(),
                profile: "default".to_owned(),
                location_id: Some("loc_123".to_owned()),
                command: "appointments.create".to_owned(),
                request_hash,
                status: IdempotencyStatus::Succeeded,
                resource_id: Some("appt_123".to_owned()),
                audit_entry_id: None,
            },
        )
        .expect("record");

        let result = create_appointment(&paths, Some("default"), None, opts).expect("replay");

        assert!(result.replayed);
        assert_eq!(result.appointment_id, Some("appt_123".to_owned()));
        assert_eq!(
            result.preflight.duplicate_check.status,
            "matched_idempotency_key"
        );
    }

    #[test]
    fn appointment_create_requires_policy_for_real_write() {
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

        let error = create_appointment(
            &paths,
            Some("default"),
            None,
            options(Some("create-appt-1"), true),
        )
        .expect_err("policy");

        assert_eq!(error.code(), "policy_denied");
    }

    fn options(
        idempotency_key: Option<&str>,
        skip_free_slot_check: bool,
    ) -> AppointmentCreateOptions {
        AppointmentCreateOptions {
            calendar_id: "cal_123".to_owned(),
            contact_id: "contact_123".to_owned(),
            starts_at: "2026-02-27T09:00:00-07:00".to_owned(),
            ends_at: "2026-02-27T09:30:00-07:00".to_owned(),
            title: Some("Discovery Call".to_owned()),
            appointment_status: AppointmentStatus::Confirmed,
            assigned_user_id: Some("user_123".to_owned()),
            address: None,
            meeting_location_type: Some("phone".to_owned()),
            timezone: Some("America/Denver".to_owned()),
            ignore_date_range: false,
            to_notify: false,
            idempotency_key: idempotency_key.map(str::to_owned),
            skip_free_slot_check,
        }
    }
}
