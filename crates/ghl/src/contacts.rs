use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

use crate::client::{AuthClass, RawGetRequest, RawPostJsonRequest, post_json, raw_get};
use crate::context::{ResolvedContext, resolve_context, resolve_context_for_dry_run};
use crate::errors::{GhlError, Result};
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
        body_json: response.body_json,
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
        body_json: response.body_json,
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
    use httpmock::Method::{GET, POST};
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
}
