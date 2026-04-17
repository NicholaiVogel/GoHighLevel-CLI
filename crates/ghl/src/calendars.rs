use chrono::{DateTime, Duration, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use url::form_urlencoded::Serializer;

use crate::client::{AuthClass, RawGetRequest, raw_get};
use crate::context::{ResolvedContext, resolve_context, resolve_context_for_dry_run};
use crate::errors::{GhlError, Result};
use crate::surfaces::Surface;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CalendarListOptions {
    pub group_id: Option<String>,
    pub show_drafted: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CalendarEventsOptions {
    pub calendar_id: Option<String>,
    pub group_id: Option<String>,
    pub user_id: Option<String>,
    pub from: Option<String>,
    pub to: Option<String>,
    pub date: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CalendarFreeSlotsOptions {
    pub calendar_id: String,
    pub date: String,
    pub timezone: Option<String>,
    pub user_id: Option<String>,
    pub enable_look_busy: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CalendarListResult {
    pub profile: String,
    pub context: ResolvedContext,
    pub location_id: String,
    pub endpoint: String,
    pub url: String,
    pub status: u16,
    pub success: bool,
    pub count: usize,
    pub calendar_ids: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub body_json: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub body_text: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CalendarListDryRun {
    pub method: &'static str,
    pub surface: &'static str,
    pub path: String,
    pub context: ResolvedContext,
    pub location_id: String,
    pub auth_class: &'static str,
    pub network: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CalendarGetResult {
    pub profile: String,
    pub context: ResolvedContext,
    pub location_id: String,
    pub calendar_id: String,
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
pub struct CalendarGetDryRun {
    pub method: &'static str,
    pub surface: &'static str,
    pub path: String,
    pub context: ResolvedContext,
    pub location_id: String,
    pub calendar_id: String,
    pub auth_class: &'static str,
    pub network: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CalendarEventsResult {
    pub profile: String,
    pub context: ResolvedContext,
    pub location_id: String,
    pub endpoint: String,
    pub url: String,
    pub status: u16,
    pub success: bool,
    pub start_time: u64,
    pub end_time: u64,
    pub count: usize,
    pub event_ids: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CalendarEventsDryRun {
    pub method: &'static str,
    pub surface: &'static str,
    pub path: String,
    pub context: ResolvedContext,
    pub location_id: String,
    pub start_time: u64,
    pub end_time: u64,
    pub auth_class: &'static str,
    pub network: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CalendarFreeSlotsResult {
    pub profile: String,
    pub context: ResolvedContext,
    pub location_id: String,
    pub calendar_id: String,
    pub endpoint: String,
    pub url: String,
    pub status: u16,
    pub success: bool,
    pub start_date: u64,
    pub end_date: u64,
    pub date_count: usize,
    pub slot_count: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub body_json: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub body_text: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CalendarFreeSlotsDryRun {
    pub method: &'static str,
    pub surface: &'static str,
    pub path: String,
    pub context: ResolvedContext,
    pub location_id: String,
    pub calendar_id: String,
    pub start_date: u64,
    pub end_date: u64,
    pub auth_class: &'static str,
    pub network: bool,
}

pub fn list_calendars(
    paths: &crate::ConfigPaths,
    profile_name: Option<&str>,
    location_override: Option<&str>,
    options: CalendarListOptions,
) -> Result<CalendarListResult> {
    validate_calendar_list_options(&options)?;
    let context = resolve_context(paths, profile_name, location_override, None)?;
    let location_id = context.require_location_id()?.to_owned();
    let endpoint = calendars_list_endpoint(&location_id, &options);
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
    let (count, calendar_ids) = summarize_id_array(response.body_json.as_ref(), "calendars");

    Ok(CalendarListResult {
        profile: context.profile.clone(),
        context,
        location_id,
        endpoint,
        url: response.url,
        status: response.status,
        success: response.success,
        count,
        calendar_ids,
        body_json: response.body_json,
        body_text: response.body_text,
    })
}

pub fn get_calendar(
    paths: &crate::ConfigPaths,
    profile_name: Option<&str>,
    location_override: Option<&str>,
    calendar_id: &str,
) -> Result<CalendarGetResult> {
    validate_calendar_id(calendar_id)?;
    let context = resolve_context(paths, profile_name, location_override, None)?;
    let location_id = context.require_location_id()?.to_owned();
    let endpoint = calendar_endpoint(calendar_id);
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

    Ok(CalendarGetResult {
        profile: context.profile.clone(),
        context,
        location_id,
        calendar_id: calendar_id.to_owned(),
        endpoint,
        url: response.url,
        status: response.status,
        success: response.success,
        body_json: response.body_json,
        body_text: response.body_text,
    })
}

pub fn list_calendar_events(
    paths: &crate::ConfigPaths,
    profile_name: Option<&str>,
    location_override: Option<&str>,
    options: CalendarEventsOptions,
) -> Result<CalendarEventsResult> {
    validate_calendar_events_options(&options)?;
    let (start_time, end_time) = event_range(&options)?;
    let context = resolve_context(paths, profile_name, location_override, None)?;
    let location_id = context.require_location_id()?.to_owned();
    let endpoint = calendar_events_endpoint(&location_id, &options, start_time, end_time);
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
    let (count, event_ids) = summarize_id_array(response.body_json.as_ref(), "events");

    Ok(CalendarEventsResult {
        profile: context.profile.clone(),
        context,
        location_id,
        endpoint,
        url: response.url,
        status: response.status,
        success: response.success,
        start_time,
        end_time,
        count,
        event_ids,
    })
}

pub fn get_calendar_free_slots(
    paths: &crate::ConfigPaths,
    profile_name: Option<&str>,
    location_override: Option<&str>,
    options: CalendarFreeSlotsOptions,
) -> Result<CalendarFreeSlotsResult> {
    validate_free_slots_options(&options)?;
    let (start_date, end_date) = date_range_ms(&options.date)?;
    let context = resolve_context(paths, profile_name, location_override, None)?;
    let location_id = context.require_location_id()?.to_owned();
    let endpoint = calendar_free_slots_endpoint(&options, start_date, end_date);
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
    let (date_count, slot_count) = summarize_slots(response.body_json.as_ref());

    Ok(CalendarFreeSlotsResult {
        profile: context.profile.clone(),
        context,
        location_id,
        calendar_id: options.calendar_id,
        endpoint,
        url: response.url,
        status: response.status,
        success: response.success,
        start_date,
        end_date,
        date_count,
        slot_count,
        body_json: response.body_json,
        body_text: response.body_text,
    })
}

pub fn calendars_list_dry_run(
    paths: &crate::ConfigPaths,
    profile_name: Option<&str>,
    location_override: Option<&str>,
    options: CalendarListOptions,
) -> Result<CalendarListDryRun> {
    validate_calendar_list_options(&options)?;
    let context = resolve_context_for_dry_run(paths, profile_name, location_override, None)?;
    let location_id = context.require_location_id()?.to_owned();

    Ok(CalendarListDryRun {
        method: "GET",
        surface: "services",
        path: calendars_list_endpoint(&location_id, &options),
        context,
        location_id,
        auth_class: "pit",
        network: false,
    })
}

pub fn get_calendar_dry_run(
    paths: &crate::ConfigPaths,
    profile_name: Option<&str>,
    location_override: Option<&str>,
    calendar_id: &str,
) -> Result<CalendarGetDryRun> {
    validate_calendar_id(calendar_id)?;
    let context = resolve_context_for_dry_run(paths, profile_name, location_override, None)?;
    let location_id = context.require_location_id()?.to_owned();

    Ok(CalendarGetDryRun {
        method: "GET",
        surface: "services",
        path: calendar_endpoint(calendar_id),
        context,
        location_id,
        calendar_id: calendar_id.to_owned(),
        auth_class: "pit",
        network: false,
    })
}

pub fn calendar_events_dry_run(
    paths: &crate::ConfigPaths,
    profile_name: Option<&str>,
    location_override: Option<&str>,
    options: CalendarEventsOptions,
) -> Result<CalendarEventsDryRun> {
    validate_calendar_events_options(&options)?;
    let (start_time, end_time) = event_range(&options)?;
    let context = resolve_context_for_dry_run(paths, profile_name, location_override, None)?;
    let location_id = context.require_location_id()?.to_owned();

    Ok(CalendarEventsDryRun {
        method: "GET",
        surface: "services",
        path: calendar_events_endpoint(&location_id, &options, start_time, end_time),
        context,
        location_id,
        start_time,
        end_time,
        auth_class: "pit",
        network: false,
    })
}

pub fn calendar_free_slots_dry_run(
    paths: &crate::ConfigPaths,
    profile_name: Option<&str>,
    location_override: Option<&str>,
    options: CalendarFreeSlotsOptions,
) -> Result<CalendarFreeSlotsDryRun> {
    validate_free_slots_options(&options)?;
    let (start_date, end_date) = date_range_ms(&options.date)?;
    let context = resolve_context_for_dry_run(paths, profile_name, location_override, None)?;
    let location_id = context.require_location_id()?.to_owned();

    Ok(CalendarFreeSlotsDryRun {
        method: "GET",
        surface: "services",
        path: calendar_free_slots_endpoint(&options, start_date, end_date),
        context,
        location_id,
        calendar_id: options.calendar_id,
        start_date,
        end_date,
        auth_class: "pit",
        network: false,
    })
}

fn calendars_list_endpoint(location_id: &str, options: &CalendarListOptions) -> String {
    let mut serializer = Serializer::new(String::new());
    serializer.append_pair("locationId", location_id);
    if let Some(group_id) = trimmed_optional(options.group_id.clone()) {
        serializer.append_pair("groupId", &group_id);
    }
    if let Some(show_drafted) = options.show_drafted {
        serializer.append_pair("showDrafted", if show_drafted { "true" } else { "false" });
    }
    format!("/calendars/?{}", serializer.finish())
}

fn calendar_endpoint(calendar_id: &str) -> String {
    format!("/calendars/{calendar_id}")
}

fn calendar_events_endpoint(
    location_id: &str,
    options: &CalendarEventsOptions,
    start_time: u64,
    end_time: u64,
) -> String {
    let mut serializer = Serializer::new(String::new());
    serializer.append_pair("locationId", location_id);
    serializer.append_pair("startTime", &start_time.to_string());
    serializer.append_pair("endTime", &end_time.to_string());
    if let Some(calendar_id) = trimmed_optional(options.calendar_id.clone()) {
        serializer.append_pair("calendarId", &calendar_id);
    }
    if let Some(group_id) = trimmed_optional(options.group_id.clone()) {
        serializer.append_pair("groupId", &group_id);
    }
    if let Some(user_id) = trimmed_optional(options.user_id.clone()) {
        serializer.append_pair("userId", &user_id);
    }
    format!("/calendars/events?{}", serializer.finish())
}

fn calendar_free_slots_endpoint(
    options: &CalendarFreeSlotsOptions,
    start_date: u64,
    end_date: u64,
) -> String {
    let mut serializer = Serializer::new(String::new());
    serializer.append_pair("startDate", &start_date.to_string());
    serializer.append_pair("endDate", &end_date.to_string());
    if let Some(timezone) = trimmed_optional(options.timezone.clone()) {
        serializer.append_pair("timezone", &timezone);
    }
    if let Some(user_id) = trimmed_optional(options.user_id.clone()) {
        serializer.append_pair("userId", &user_id);
    }
    if options.enable_look_busy {
        serializer.append_pair("enableLookBusy", "true");
    }
    format!(
        "/calendars/{}/free-slots?{}",
        options.calendar_id,
        serializer.finish()
    )
}

fn event_range(options: &CalendarEventsOptions) -> Result<(u64, u64)> {
    match (&options.date, &options.from, &options.to) {
        (Some(date), None, None) => date_range_ms(date),
        (None, Some(from), Some(to)) => {
            let start = parse_timestamp_ms(from, "calendar event --from")?;
            let end = parse_timestamp_ms(to, "calendar event --to")?;
            validate_range(start, end, "calendar event range")?;
            Ok((start, end))
        }
        (Some(_), Some(_), _) | (Some(_), _, Some(_)) => Err(GhlError::Validation {
            message: "pass either --date or --from/--to for calendar events, not both".to_owned(),
        }),
        _ => Err(GhlError::Validation {
            message: "calendar events require --date or both --from and --to".to_owned(),
        }),
    }
}

fn date_range_ms(date: &str) -> Result<(u64, u64)> {
    let date =
        NaiveDate::parse_from_str(date.trim(), "%Y-%m-%d").map_err(|_| GhlError::Validation {
            message: "date must use YYYY-MM-DD format".to_owned(),
        })?;
    let start = date
        .and_hms_opt(0, 0, 0)
        .ok_or_else(|| GhlError::Validation {
            message: "date could not be converted to a start time".to_owned(),
        })?
        .and_utc();
    let end = start + Duration::days(1);
    let start = millis_to_u64(start.timestamp_millis(), "date start")?;
    let end = millis_to_u64(end.timestamp_millis(), "date end")?;
    validate_range(start, end, "date range")?;
    Ok((start, end))
}

fn parse_timestamp_ms(value: &str, label: &str) -> Result<u64> {
    let value = value.trim();
    if value.is_empty() {
        return Err(GhlError::Validation {
            message: format!("{label} cannot be empty"),
        });
    }
    if value.chars().all(|character| character.is_ascii_digit()) {
        return value.parse::<u64>().map_err(|_| GhlError::Validation {
            message: format!("{label} must be a valid epoch-millisecond value"),
        });
    }
    let parsed = DateTime::parse_from_rfc3339(value).map_err(|_| GhlError::Validation {
        message: format!("{label} must be epoch milliseconds or RFC3339 datetime"),
    })?;
    millis_to_u64(
        parsed.with_timezone(&Utc).timestamp_millis(),
        "RFC3339 datetime",
    )
}

fn millis_to_u64(value: i64, label: &str) -> Result<u64> {
    u64::try_from(value).map_err(|_| GhlError::Validation {
        message: format!("{label} must not be before the Unix epoch"),
    })
}

fn validate_range(start: u64, end: u64, label: &str) -> Result<()> {
    if end <= start {
        return Err(GhlError::Validation {
            message: format!("{label} end must be after start"),
        });
    }
    Ok(())
}

fn summarize_id_array(body: Option<&Value>, field: &str) -> (usize, Vec<String>) {
    let items = body
        .and_then(|body| body.get(field))
        .and_then(Value::as_array)
        .map(Vec::as_slice)
        .unwrap_or(&[]);
    let ids = items
        .iter()
        .filter_map(|item| item.get("id").and_then(Value::as_str).map(str::to_owned))
        .collect::<Vec<_>>();
    (items.len(), ids)
}

fn summarize_slots(body: Option<&Value>) -> (usize, usize) {
    let Some(Value::Object(map)) = body else {
        return (0, 0);
    };
    let slot_count = map
        .values()
        .filter_map(|value| value.get("slots").and_then(Value::as_array))
        .map(Vec::len)
        .sum();
    (map.len(), slot_count)
}

fn validate_calendar_list_options(options: &CalendarListOptions) -> Result<()> {
    validate_optional_text(options.group_id.as_deref(), "calendar group id")
}

fn validate_calendar_events_options(options: &CalendarEventsOptions) -> Result<()> {
    validate_optional_segment(options.calendar_id.as_deref(), "calendar id")?;
    validate_optional_text(options.group_id.as_deref(), "calendar group id")?;
    validate_optional_text(options.user_id.as_deref(), "calendar user id")?;
    Ok(())
}

fn validate_free_slots_options(options: &CalendarFreeSlotsOptions) -> Result<()> {
    validate_calendar_id(&options.calendar_id)?;
    validate_optional_text(options.timezone.as_deref(), "calendar timezone")?;
    validate_optional_text(options.user_id.as_deref(), "calendar user id")?;
    Ok(())
}

fn validate_calendar_id(calendar_id: &str) -> Result<()> {
    validate_path_segment(calendar_id, "calendar id")
}

fn validate_optional_segment(value: Option<&str>, label: &str) -> Result<()> {
    if let Some(value) = value {
        validate_path_segment(value, label)?;
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
    if value.is_some_and(|value| value.trim().is_empty()) {
        return Err(GhlError::Validation {
            message: format!("{label} cannot be empty"),
        });
    }
    Ok(())
}

fn trimmed_optional(value: Option<String>) -> Option<String> {
    value.and_then(|value| {
        let trimmed = value.trim();
        if trimmed.is_empty() {
            None
        } else {
            Some(trimmed.to_owned())
        }
    })
}

#[cfg(test)]
mod tests {
    use httpmock::Method::GET;
    use httpmock::MockServer;
    use serde_json::json;

    use super::*;
    use crate::auth::add_pit;
    use crate::config::resolve_paths;

    #[test]
    fn calendars_list_uses_location_context_and_filters() {
        let server = MockServer::start();
        let mock = server.mock(|when, then| {
            when.method(GET)
                .path("/calendars/")
                .query_param("locationId", "loc_123")
                .query_param("groupId", "group_123")
                .query_param("showDrafted", "false")
                .header("authorization", "Bearer pit-secret");
            then.status(200).json_body(json!({
                "calendars": [
                    { "id": "cal_123", "name": "Discovery" }
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
        point_services_base_url(&paths, &server);

        let result = list_calendars(
            &paths,
            Some("default"),
            None,
            CalendarListOptions {
                group_id: Some("group_123".to_owned()),
                show_drafted: Some(false),
            },
        )
        .expect("calendars");

        mock.assert();
        assert_eq!(result.location_id, "loc_123");
        assert_eq!(result.count, 1);
        assert_eq!(result.calendar_ids, vec!["cal_123".to_owned()]);
    }

    #[test]
    fn calendar_get_rejects_path_injection() {
        let temp = tempfile::tempdir().expect("tempdir");
        let paths = resolve_paths(Some(temp.path()));

        let error = get_calendar_dry_run(&paths, None, Some("loc_123"), "../cal_123")
            .expect_err("invalid id");

        assert_eq!(error.code(), "validation_error");
    }

    #[test]
    fn calendar_events_uses_date_range_and_returns_summary_only() {
        let server = MockServer::start();
        let mock = server.mock(|when, then| {
            when.method(GET)
                .path("/calendars/events")
                .query_param("locationId", "loc_123")
                .query_param("calendarId", "cal_123")
                .query_param("startTime", "1772150400000")
                .query_param("endTime", "1772236800000");
            then.status(200).json_body(json!({
                "events": [
                    { "id": "evt_123", "title": "Private appointment", "notes": "secret" }
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
        point_services_base_url(&paths, &server);

        let result = list_calendar_events(
            &paths,
            Some("default"),
            None,
            CalendarEventsOptions {
                calendar_id: Some("cal_123".to_owned()),
                group_id: None,
                user_id: None,
                from: None,
                to: None,
                date: Some("2026-02-27".to_owned()),
            },
        )
        .expect("events");

        mock.assert();
        assert_eq!(result.count, 1);
        assert_eq!(result.event_ids, vec!["evt_123".to_owned()]);
    }

    #[test]
    fn calendar_free_slots_uses_date_range_and_counts_slots() {
        let server = MockServer::start();
        let mock = server.mock(|when, then| {
            when.method(GET)
                .path("/calendars/cal_123/free-slots")
                .query_param("startDate", "1772150400000")
                .query_param("endDate", "1772236800000")
                .query_param("timezone", "America/Denver")
                .query_param("enableLookBusy", "true");
            then.status(200).json_body(json!({
                "2026-02-27": { "slots": ["2026-02-27T16:00:00Z", "2026-02-27T16:30:00Z"] }
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

        let result = get_calendar_free_slots(
            &paths,
            Some("default"),
            None,
            CalendarFreeSlotsOptions {
                calendar_id: "cal_123".to_owned(),
                date: "2026-02-27".to_owned(),
                timezone: Some("America/Denver".to_owned()),
                user_id: None,
                enable_look_busy: true,
            },
        )
        .expect("slots");

        mock.assert();
        assert_eq!(result.date_count, 1);
        assert_eq!(result.slot_count, 2);
    }

    #[test]
    fn calendar_events_rejects_mixed_date_and_range() {
        let options = CalendarEventsOptions {
            calendar_id: None,
            group_id: None,
            user_id: None,
            from: Some("2026-02-27T00:00:00Z".to_owned()),
            to: Some("2026-02-28T00:00:00Z".to_owned()),
            date: Some("2026-02-27".to_owned()),
        };

        let error = event_range(&options).expect_err("mixed range");

        assert_eq!(error.code(), "validation_error");
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
}
