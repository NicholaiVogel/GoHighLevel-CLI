use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use url::form_urlencoded::Serializer;

use crate::client::{AuthClass, RawGetRequest, RawPostJsonRequest, post_json, raw_get};
use crate::context::{ResolvedContext, resolve_context, resolve_context_for_dry_run};
use crate::errors::{GhlError, Result};
use crate::surfaces::Surface;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct UserListOptions {
    pub skip: u32,
    pub limit: u32,
    pub user_type: Option<String>,
    pub role: Option<String>,
    pub ids: Option<String>,
    pub sort: Option<String>,
    pub sort_direction: Option<UserSortDirection>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum UserSortDirection {
    Asc,
    Desc,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct UserSearchOptions {
    pub query: Option<String>,
    pub email: Option<String>,
    pub skip: u32,
    pub limit: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct UserListResult {
    pub profile: String,
    pub context: ResolvedContext,
    pub location_id: String,
    pub skip: u32,
    pub limit: u32,
    pub endpoint: String,
    pub url: String,
    pub status: u16,
    pub success: bool,
    pub count: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub total: Option<u64>,
    pub user_ids: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct UserListDryRun {
    pub method: &'static str,
    pub surface: &'static str,
    pub path: String,
    pub context: ResolvedContext,
    pub location_id: String,
    pub auth_class: &'static str,
    pub network: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct UserGetResult {
    pub profile: String,
    pub context: ResolvedContext,
    pub location_id: String,
    pub user_id: String,
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
pub struct UserGetDryRun {
    pub method: &'static str,
    pub surface: &'static str,
    pub path: String,
    pub context: ResolvedContext,
    pub location_id: String,
    pub user_id: String,
    pub auth_class: &'static str,
    pub network: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct UserSearchResult {
    pub profile: String,
    pub context: ResolvedContext,
    pub search_mode: UserSearchMode,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub location_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub company_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub query: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub email_filter: Option<String>,
    pub skip: u32,
    pub limit: u32,
    pub endpoint: String,
    pub url: String,
    pub status: u16,
    pub success: bool,
    pub count: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub total: Option<u64>,
    pub user_ids: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct UserSearchDryRun {
    pub method: &'static str,
    pub surface: &'static str,
    pub path: String,
    pub context: ResolvedContext,
    pub search_mode: UserSearchMode,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub location_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub company_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub request_body_json: Option<Value>,
    pub auth_class: &'static str,
    pub network: bool,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum UserSearchMode {
    Query,
    Email,
}

pub fn list_users(
    paths: &crate::ConfigPaths,
    profile_name: Option<&str>,
    location_override: Option<&str>,
    options: UserListOptions,
) -> Result<UserListResult> {
    validate_user_list_options(&options)?;
    let context = resolve_context(paths, profile_name, location_override, None)?;
    let location_id = context.require_location_id()?.to_owned();
    let endpoint = users_list_endpoint(&location_id);
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
    let (count, total, user_ids) =
        summarize_users_window(response.body_json.as_ref(), options.skip, options.limit);

    Ok(UserListResult {
        profile: context.profile.clone(),
        context,
        location_id,
        skip: options.skip,
        limit: options.limit,
        endpoint,
        url: response.url,
        status: response.status,
        success: response.success,
        count,
        total,
        user_ids,
    })
}

pub fn users_list_dry_run(
    paths: &crate::ConfigPaths,
    profile_name: Option<&str>,
    location_override: Option<&str>,
    options: UserListOptions,
) -> Result<UserListDryRun> {
    validate_user_list_options(&options)?;
    let context = resolve_context_for_dry_run(paths, profile_name, location_override, None)?;
    let location_id = context.require_location_id()?.to_owned();

    Ok(UserListDryRun {
        method: "GET",
        surface: "services",
        path: users_list_endpoint(&location_id),
        context,
        location_id,
        auth_class: "pit",
        network: false,
    })
}

pub fn get_user(
    paths: &crate::ConfigPaths,
    profile_name: Option<&str>,
    location_override: Option<&str>,
    user_id: &str,
) -> Result<UserGetResult> {
    validate_user_id(user_id)?;
    let context = resolve_context(paths, profile_name, location_override, None)?;
    let location_id = context.require_location_id()?.to_owned();
    let endpoint = user_get_endpoint(user_id);
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

    Ok(UserGetResult {
        profile: context.profile.clone(),
        context,
        location_id,
        user_id: user_id.to_owned(),
        endpoint,
        url: response.url,
        status: response.status,
        success: response.success,
        body_json: response.body_json,
        body_text: response.body_text,
    })
}

pub fn get_user_dry_run(
    paths: &crate::ConfigPaths,
    profile_name: Option<&str>,
    location_override: Option<&str>,
    user_id: &str,
) -> Result<UserGetDryRun> {
    validate_user_id(user_id)?;
    let context = resolve_context_for_dry_run(paths, profile_name, location_override, None)?;
    let location_id = context.require_location_id()?.to_owned();

    Ok(UserGetDryRun {
        method: "GET",
        surface: "services",
        path: user_get_endpoint(user_id),
        context,
        location_id,
        user_id: user_id.to_owned(),
        auth_class: "pit",
        network: false,
    })
}

pub fn search_users(
    paths: &crate::ConfigPaths,
    profile_name: Option<&str>,
    location_override: Option<&str>,
    company_override: Option<&str>,
    options: UserSearchOptions,
) -> Result<UserSearchResult> {
    validate_user_search_options(&options)?;
    if let Some(email) = trimmed_optional(options.email.clone()) {
        search_users_by_email(paths, profile_name, location_override, options, email)
    } else {
        search_users_by_query(paths, profile_name, company_override, options)
    }
}

pub fn users_search_dry_run(
    paths: &crate::ConfigPaths,
    profile_name: Option<&str>,
    location_override: Option<&str>,
    company_override: Option<&str>,
    options: UserSearchOptions,
) -> Result<UserSearchDryRun> {
    validate_user_search_options(&options)?;
    if let Some(email) = trimmed_optional(options.email.clone()) {
        let context = resolve_context_for_dry_run(paths, profile_name, location_override, None)?;
        let location_id = context.require_location_id()?.to_owned();
        return Ok(UserSearchDryRun {
            method: "POST",
            surface: "services",
            path: users_filter_by_email_endpoint().to_owned(),
            context,
            search_mode: UserSearchMode::Email,
            location_id: Some(location_id.clone()),
            company_id: None,
            request_body_json: Some(users_filter_by_email_body(&location_id, &email)),
            auth_class: "pit",
            network: false,
        });
    }

    let context = resolve_context_for_dry_run(paths, profile_name, None, company_override)?;
    let company_id = context.require_company_id()?.to_owned();
    Ok(UserSearchDryRun {
        method: "GET",
        surface: "services",
        path: users_search_endpoint(&company_id, &options),
        context,
        search_mode: UserSearchMode::Query,
        location_id: None,
        company_id: Some(company_id),
        request_body_json: None,
        auth_class: "pit",
        network: false,
    })
}

fn search_users_by_email(
    paths: &crate::ConfigPaths,
    profile_name: Option<&str>,
    location_override: Option<&str>,
    options: UserSearchOptions,
    email: String,
) -> Result<UserSearchResult> {
    let context = resolve_context(paths, profile_name, location_override, None)?;
    let location_id = context.require_location_id()?.to_owned();
    let endpoint = users_filter_by_email_endpoint().to_owned();
    let response = post_json(
        paths,
        profile_name,
        RawPostJsonRequest {
            surface: Surface::Services,
            path: endpoint.clone(),
            auth_class: AuthClass::Pit,
            body: users_filter_by_email_body(&location_id, &email),
            include_body: true,
        },
    )?;
    let (count, total, user_ids) = summarize_users(response.body_json.as_ref());

    Ok(UserSearchResult {
        profile: context.profile.clone(),
        context,
        search_mode: UserSearchMode::Email,
        location_id: Some(location_id),
        company_id: None,
        query: None,
        email_filter: Some(email),
        skip: options.skip,
        limit: options.limit,
        endpoint,
        url: response.url,
        status: response.status,
        success: response.success,
        count,
        total,
        user_ids,
    })
}

fn search_users_by_query(
    paths: &crate::ConfigPaths,
    profile_name: Option<&str>,
    company_override: Option<&str>,
    options: UserSearchOptions,
) -> Result<UserSearchResult> {
    let query = trimmed_optional(options.query.clone()).expect("validated query");
    let context = resolve_context(paths, profile_name, None, company_override)?;
    let company_id = context.require_company_id()?.to_owned();
    let endpoint = users_search_endpoint(&company_id, &options);
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
    let (count, total, user_ids) = summarize_users(response.body_json.as_ref());

    Ok(UserSearchResult {
        profile: context.profile.clone(),
        context,
        search_mode: UserSearchMode::Query,
        location_id: None,
        company_id: Some(company_id),
        query: Some(query),
        email_filter: None,
        skip: options.skip,
        limit: options.limit,
        endpoint,
        url: response.url,
        status: response.status,
        success: response.success,
        count,
        total,
        user_ids,
    })
}

fn users_list_endpoint(location_id: &str) -> String {
    let mut serializer = Serializer::new(String::new());
    serializer.append_pair("locationId", location_id);

    format!("/users/?{}", serializer.finish())
}

fn user_get_endpoint(user_id: &str) -> String {
    format!("/users/{user_id}")
}

fn users_search_endpoint(company_id: &str, options: &UserSearchOptions) -> String {
    let mut serializer = Serializer::new(String::new());
    serializer.append_pair("companyId", company_id);
    if let Some(query) = trimmed_optional_ref(options.query.as_deref()) {
        serializer.append_pair("query", query);
    }
    serializer.append_pair("skip", &options.skip.to_string());
    serializer.append_pair("limit", &options.limit.to_string());

    format!("/users/search?{}", serializer.finish())
}

fn users_filter_by_email_endpoint() -> &'static str {
    "/users/search/filter-by-email"
}

fn users_filter_by_email_body(location_id: &str, email: &str) -> Value {
    let mut body = Map::new();
    body.insert("email".to_owned(), Value::String(email.to_owned()));
    body.insert(
        "locationId".to_owned(),
        Value::String(location_id.to_owned()),
    );
    Value::Object(body)
}

fn summarize_users(body: Option<&Value>) -> (usize, Option<u64>, Vec<String>) {
    let Some(body) = body else {
        return (0, None, Vec::new());
    };
    let users = user_array(body);
    let user_ids = users
        .iter()
        .filter_map(|user| {
            user.get("id")
                .or_else(|| user.get("_id"))
                .or_else(|| user.get("userId"))
                .and_then(Value::as_str)
                .map(str::to_owned)
        })
        .collect::<Vec<_>>();
    let total = body
        .get("total")
        .and_then(Value::as_u64)
        .or_else(|| body.get("count").and_then(Value::as_u64))
        .or_else(|| {
            body.get("meta")
                .and_then(|meta| meta.get("total"))
                .and_then(Value::as_u64)
        });

    (users.len(), total, user_ids)
}

fn summarize_users_window(
    body: Option<&Value>,
    skip: u32,
    limit: u32,
) -> (usize, Option<u64>, Vec<String>) {
    let (available_count, total, user_ids) = summarize_users(body);
    let skip = skip as usize;
    let limit = limit as usize;
    let visible_count = available_count.saturating_sub(skip).min(limit);
    let visible_ids = user_ids
        .into_iter()
        .skip(skip)
        .take(limit)
        .collect::<Vec<_>>();
    let total = total.or_else(|| {
        (available_count > visible_count || skip > 0).then_some(available_count as u64)
    });

    (visible_count, total, visible_ids)
}

fn user_array(body: &Value) -> &[Value] {
    if let Some(users) = body.as_array() {
        return users.as_slice();
    }
    for field in ["users", "members", "team", "teamMembers", "data"] {
        if let Some(users) = body.get(field).and_then(Value::as_array) {
            return users.as_slice();
        }
    }
    &[]
}

fn validate_user_id(user_id: &str) -> Result<()> {
    validate_path_segment(user_id, "user id")
}

fn validate_user_list_options(options: &UserListOptions) -> Result<()> {
    validate_limit(options.limit, "user list")?;
    validate_optional_text(options.user_type.as_deref(), "user type")?;
    validate_optional_text(options.role.as_deref(), "user role")?;
    validate_optional_text(options.ids.as_deref(), "user ids")?;
    validate_optional_text(options.sort.as_deref(), "user sort")?;
    if has_value(options.user_type.as_deref())
        || has_value(options.role.as_deref())
        || has_value(options.ids.as_deref())
        || has_value(options.sort.as_deref())
        || options.sort_direction.is_some()
    {
        return Err(GhlError::Validation {
            message: "users list currently supports location context plus client-side --skip and --limit only".to_owned(),
        });
    }

    Ok(())
}

fn validate_user_search_options(options: &UserSearchOptions) -> Result<()> {
    let has_query = has_value(options.query.as_deref());
    let has_email = has_value(options.email.as_deref());
    if has_query == has_email {
        return Err(GhlError::Validation {
            message: "user search needs exactly one of --query or --email".to_owned(),
        });
    }
    validate_limit(options.limit, "user search")?;
    validate_optional_text(options.query.as_deref(), "user search query")?;
    validate_optional_text(options.email.as_deref(), "user email filter")?;

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

fn trimmed_optional_ref(value: Option<&str>) -> Option<&str> {
    value.map(str::trim).filter(|value| !value.is_empty())
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
    fn users_list_uses_location_context_and_returns_summary_only() {
        let server = MockServer::start();
        let mock = server.mock(|when, then| {
            when.method(GET)
                .path("/users/")
                .query_param("locationId", "loc_123");
            then.status(200)
                .header("content-type", "application/json")
                .json_body(json!({
                    "users": [
                        { "id": "user_123", "email": "person@example.com", "role": "admin" },
                        { "id": "user_456", "email": "other@example.com", "role": "member" }
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

        let result = list_users(
            &paths,
            Some("default"),
            None,
            UserListOptions {
                skip: 0,
                limit: 1,
                user_type: None,
                role: None,
                ids: None,
                sort: None,
                sort_direction: None,
            },
        )
        .expect("users");

        mock.assert();
        assert_eq!(result.location_id, "loc_123");
        assert_eq!(result.count, 1);
        assert_eq!(result.total, Some(2));
        assert_eq!(result.user_ids, vec!["user_123".to_owned()]);
    }

    #[test]
    fn users_get_requires_location_context_and_redacts_body() {
        let server = MockServer::start();
        let mock = server.mock(|when, then| {
            when.method(GET)
                .path("/users/user_123")
                .header("authorization", "Bearer pit-secret");
            then.status(200)
                .header("content-type", "application/json")
                .json_body(json!({
                    "user": {
                        "id": "user_123",
                        "email": "person@example.com",
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
        point_services_base_url(&paths, &server);

        let result = get_user(&paths, Some("default"), None, "user_123").expect("user");

        mock.assert();
        assert_eq!(result.location_id, "loc_123");
        assert_eq!(result.status, 200);
        assert_eq!(result.body_json.unwrap()["user"]["apiKey"], "[REDACTED]");
    }

    #[test]
    fn users_search_by_query_uses_company_context() {
        let server = MockServer::start();
        let mock = server.mock(|when, then| {
            when.method(GET)
                .path("/users/search")
                .query_param("companyId", "company_123")
                .query_param("query", "Person")
                .query_param("skip", "0")
                .query_param("limit", "10");
            then.status(200)
                .header("content-type", "application/json")
                .json_body(json!({
                    "users": [{ "id": "user_123", "name": "Person" }],
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
            Some("company_123".to_owned()),
            true,
        )
        .expect("pit");
        point_services_base_url(&paths, &server);

        let result = search_users(
            &paths,
            Some("default"),
            None,
            None,
            UserSearchOptions {
                query: Some("Person".to_owned()),
                email: None,
                skip: 0,
                limit: 10,
            },
        )
        .expect("users");

        mock.assert();
        assert_eq!(result.search_mode, UserSearchMode::Query);
        assert_eq!(result.company_id.as_deref(), Some("company_123"));
        assert_eq!(result.count, 1);
        assert_eq!(result.user_ids, vec!["user_123".to_owned()]);
    }

    #[test]
    fn users_search_by_email_uses_location_context() {
        let server = MockServer::start();
        let mock = server.mock(|when, then| {
            when.method(POST)
                .path("/users/search/filter-by-email")
                .json_body(json!({
                    "email": "person@example.com",
                    "locationId": "loc_123"
                }));
            then.status(200)
                .header("content-type", "application/json")
                .json_body(json!({
                    "users": [{ "id": "user_123", "email": "person@example.com" }]
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

        let result = search_users(
            &paths,
            Some("default"),
            None,
            None,
            UserSearchOptions {
                query: None,
                email: Some("person@example.com".to_owned()),
                skip: 0,
                limit: 25,
            },
        )
        .expect("users");

        mock.assert();
        assert_eq!(result.search_mode, UserSearchMode::Email);
        assert_eq!(result.location_id.as_deref(), Some("loc_123"));
        assert_eq!(result.count, 1);
        assert_eq!(result.user_ids, vec!["user_123".to_owned()]);
    }

    #[test]
    fn users_search_dry_run_uses_location_override_for_email_without_profile() {
        let temp = tempfile::tempdir().expect("tempdir");
        let paths = resolve_paths(Some(temp.path()));

        let result = users_search_dry_run(
            &paths,
            Some("missing"),
            Some("loc_override"),
            None,
            UserSearchOptions {
                query: None,
                email: Some("person@example.com".to_owned()),
                skip: 0,
                limit: 25,
            },
        )
        .expect("dry run");

        assert_eq!(result.method, "POST");
        assert_eq!(result.path, "/users/search/filter-by-email");
        assert_eq!(result.search_mode, UserSearchMode::Email);
        assert_eq!(result.location_id.as_deref(), Some("loc_override"));
        assert_eq!(
            result.request_body_json.unwrap()["email"],
            "person@example.com"
        );
    }

    #[test]
    fn users_search_dry_run_uses_company_override_for_query_without_profile() {
        let temp = tempfile::tempdir().expect("tempdir");
        let paths = resolve_paths(Some(temp.path()));

        let result = users_search_dry_run(
            &paths,
            Some("missing"),
            None,
            Some("company_123"),
            UserSearchOptions {
                query: Some("Person".to_owned()),
                email: None,
                skip: 5,
                limit: 10,
            },
        )
        .expect("dry run");

        assert_eq!(result.method, "GET");
        assert_eq!(
            result.path,
            "/users/search?companyId=company_123&query=Person&skip=5&limit=10"
        );
        assert_eq!(result.search_mode, UserSearchMode::Query);
        assert_eq!(result.company_id.as_deref(), Some("company_123"));
    }

    #[test]
    fn users_search_requires_exactly_one_query_mode() {
        let temp = tempfile::tempdir().expect("tempdir");
        let paths = resolve_paths(Some(temp.path()));

        let error = users_search_dry_run(
            &paths,
            None,
            Some("loc_123"),
            Some("company_123"),
            UserSearchOptions {
                query: Some("Person".to_owned()),
                email: Some("person@example.com".to_owned()),
                skip: 0,
                limit: 25,
            },
        )
        .expect_err("conflicting search modes");

        assert_eq!(error.code(), "validation_error");
    }

    #[test]
    fn users_get_rejects_path_injection() {
        let temp = tempfile::tempdir().expect("tempdir");
        let paths = resolve_paths(Some(temp.path()));

        let error =
            get_user_dry_run(&paths, None, Some("loc_123"), "../user_123").expect_err("invalid id");

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
