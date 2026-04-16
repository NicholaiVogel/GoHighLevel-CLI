use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::client::{AuthClass, RawGetRequest, raw_get};
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

fn location_endpoint(location_id: &str) -> String {
    format!("/locations/{location_id}")
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
}
