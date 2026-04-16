use serde::{Deserialize, Serialize};
use serde_json::Value;
use url::form_urlencoded::Serializer;

use crate::client::{AuthClass, RawGetRequest, raw_get};
use crate::context::{ResolvedContext, resolve_context, resolve_context_for_dry_run};
use crate::errors::{GhlError, Result};
use crate::profiles::load_profiles;
use crate::surfaces::Surface;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct LocationGetResult {
    pub profile: String,
    pub location_id: String,
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
pub struct LocationGetDryRun {
    pub method: &'static str,
    pub surface: &'static str,
    pub path: String,
    pub location_id: String,
    pub auth_class: &'static str,
    pub network: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct LocationSearchOptions {
    pub company_id: Option<String>,
    pub email: Option<String>,
    pub skip: u32,
    pub limit: u32,
    pub order: LocationSearchOrder,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum LocationSearchOrder {
    Asc,
    Desc,
}

impl LocationSearchOrder {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Asc => "asc",
            Self::Desc => "desc",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct LocationSearchResult {
    pub profile: String,
    pub context: ResolvedContext,
    pub company_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub email_filter: Option<String>,
    pub endpoint: String,
    pub url: String,
    pub status: u16,
    pub success: bool,
    pub skip: u32,
    pub limit: u32,
    pub order: LocationSearchOrder,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub body_json: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub body_text: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct LocationSearchDryRun {
    pub method: &'static str,
    pub surface: &'static str,
    pub path: String,
    pub context: ResolvedContext,
    pub company_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub email_filter: Option<String>,
    pub skip: u32,
    pub limit: u32,
    pub order: LocationSearchOrder,
    pub auth_class: &'static str,
    pub network: bool,
}

pub fn get_location(
    paths: &crate::ConfigPaths,
    profile_name: Option<&str>,
    location_id: &str,
) -> Result<LocationGetResult> {
    validate_location_id(location_id)?;
    let profiles = load_profiles(paths)?;
    let selected_profile = profiles.selected_name(profile_name).to_owned();
    let endpoint = location_endpoint(location_id);
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

    Ok(LocationGetResult {
        profile: selected_profile,
        location_id: location_id.to_owned(),
        endpoint,
        url: response.url,
        status: response.status,
        success: response.success,
        body_json: response.body_json,
        body_text: response.body_text,
    })
}

pub fn list_locations(
    paths: &crate::ConfigPaths,
    profile_name: Option<&str>,
    location_override: Option<&str>,
    options: LocationSearchOptions,
) -> Result<LocationSearchResult> {
    search_locations(paths, profile_name, location_override, options)
}

pub fn search_locations(
    paths: &crate::ConfigPaths,
    profile_name: Option<&str>,
    location_override: Option<&str>,
    options: LocationSearchOptions,
) -> Result<LocationSearchResult> {
    validate_search_options(&options)?;
    let context = resolve_context(
        paths,
        profile_name,
        location_override,
        options.company_id.as_deref(),
    )?;
    let company_id = context.require_company_id()?.to_owned();
    let endpoint = locations_search_endpoint(&company_id, &options);
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

    Ok(LocationSearchResult {
        profile: context.profile.clone(),
        context,
        company_id,
        email_filter: options.email,
        endpoint,
        url: response.url,
        status: response.status,
        success: response.success,
        skip: options.skip,
        limit: options.limit,
        order: options.order,
        body_json: response.body_json,
        body_text: response.body_text,
    })
}

pub fn get_location_dry_run(location_id: &str) -> Result<LocationGetDryRun> {
    validate_location_id(location_id)?;
    Ok(LocationGetDryRun {
        method: "GET",
        surface: "services",
        path: location_endpoint(location_id),
        location_id: location_id.to_owned(),
        auth_class: "pit",
        network: false,
    })
}

pub fn locations_search_dry_run(
    paths: &crate::ConfigPaths,
    profile_name: Option<&str>,
    location_override: Option<&str>,
    options: LocationSearchOptions,
) -> Result<LocationSearchDryRun> {
    validate_search_options(&options)?;
    let context = resolve_context_for_dry_run(
        paths,
        profile_name,
        location_override,
        options.company_id.as_deref(),
    )?;
    let company_id = context.require_company_id()?.to_owned();

    Ok(LocationSearchDryRun {
        method: "GET",
        surface: "services",
        path: locations_search_endpoint(&company_id, &options),
        context,
        company_id,
        email_filter: options.email,
        skip: options.skip,
        limit: options.limit,
        order: options.order,
        auth_class: "pit",
        network: false,
    })
}

fn location_endpoint(location_id: &str) -> String {
    format!("/locations/{location_id}")
}

fn locations_search_endpoint(company_id: &str, options: &LocationSearchOptions) -> String {
    let mut serializer = Serializer::new(String::new());
    serializer.append_pair("companyId", company_id);
    serializer.append_pair("skip", &options.skip.to_string());
    serializer.append_pair("limit", &options.limit.to_string());
    serializer.append_pair("order", options.order.as_str());
    if let Some(email) = &options.email {
        serializer.append_pair("email", email);
    }
    format!("/locations/search?{}", serializer.finish())
}

fn validate_location_id(location_id: &str) -> Result<()> {
    if location_id.trim().is_empty() {
        return Err(GhlError::Validation {
            message: "location id cannot be empty".to_owned(),
        });
    }
    if location_id.chars().any(|character| {
        character == '/'
            || character == '?'
            || character == '#'
            || character.is_ascii_control()
            || character.is_whitespace()
    }) {
        return Err(GhlError::Validation {
            message: "location id must be a single path segment".to_owned(),
        });
    }

    Ok(())
}

fn validate_company_id(company_id: &str) -> Result<()> {
    if company_id.trim().is_empty() {
        return Err(GhlError::AmbiguousContext {
            context: "company".to_owned(),
            message: "company context is required; pass --company or set a profile company id"
                .to_owned(),
        });
    }
    if company_id.chars().any(|character| {
        character == '/'
            || character == '?'
            || character == '#'
            || character.is_ascii_control()
            || character.is_whitespace()
    }) {
        return Err(GhlError::Validation {
            message: "company id must be a single identifier without whitespace".to_owned(),
        });
    }

    Ok(())
}

fn validate_search_options(options: &LocationSearchOptions) -> Result<()> {
    if let Some(company_id) = &options.company_id {
        validate_company_id(company_id)?;
    }
    if let Some(email) = &options.email {
        if email.trim().is_empty() {
            return Err(GhlError::Validation {
                message: "location search query cannot be empty".to_owned(),
            });
        }
        if email.chars().any(|character| character.is_ascii_control()) {
            return Err(GhlError::Validation {
                message: "location search query cannot contain control characters".to_owned(),
            });
        }
    }
    if options.limit == 0 || options.limit > 200 {
        return Err(GhlError::Validation {
            message: "location search limit must be between 1 and 200".to_owned(),
        });
    }

    Ok(())
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
    fn location_get_returns_redacted_body() {
        let server = MockServer::start();
        let mock = server.mock(|when, then| {
            when.method(GET)
                .path("/locations/loc_123")
                .header("authorization", "Bearer pit-secret");
            then.status(200)
                .header("content-type", "application/json")
                .json_body(json!({
                    "id": "loc_123",
                    "name": "Test Location",
                    "apiKey": "secret-value"
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

        let result = get_location(&paths, Some("default"), "loc_123").expect("location");

        mock.assert();
        assert_eq!(result.status, 200);
        assert_eq!(result.body_json.unwrap()["apiKey"], "[REDACTED]");
    }

    #[test]
    fn location_get_rejects_path_injection() {
        let error = get_location_dry_run("../loc_123").expect_err("invalid id");

        assert_eq!(error.code(), "validation_error");
    }

    #[test]
    fn locations_search_uses_company_context_and_email_filter() {
        let server = MockServer::start();
        let mock = server.mock(|when, then| {
            when.method(GET)
                .path("/locations/search")
                .query_param("companyId", "company_123")
                .query_param("email", "test@example.com")
                .query_param("limit", "50")
                .query_param("skip", "0")
                .query_param("order", "asc");
            then.status(200)
                .header("content-type", "application/json")
                .json_body(json!({
                    "locations": [
                        { "_id": "loc_123", "name": "Test Location", "apiKey": "secret-value" }
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
            Some("company_123".to_owned()),
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

        let result = search_locations(
            &paths,
            Some("default"),
            None,
            LocationSearchOptions {
                company_id: None,
                email: Some("test@example.com".to_owned()),
                skip: 0,
                limit: 50,
                order: LocationSearchOrder::Asc,
            },
        )
        .expect("locations");

        mock.assert();
        assert_eq!(result.company_id, "company_123");
        assert_eq!(
            result.body_json.unwrap()["locations"][0]["apiKey"],
            "[REDACTED]"
        );
    }

    #[test]
    fn locations_search_dry_run_requires_company_context() {
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

        let error = locations_search_dry_run(
            &paths,
            Some("default"),
            None,
            LocationSearchOptions {
                company_id: None,
                email: None,
                skip: 0,
                limit: 50,
                order: LocationSearchOrder::Asc,
            },
        )
        .expect_err("missing company");

        assert_eq!(error.code(), "ambiguous_context");
    }
}
