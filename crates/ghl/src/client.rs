use std::collections::BTreeMap;
use std::time::Duration;

use reqwest::blocking::Client;
use reqwest::header::{
    ACCEPT, AUTHORIZATION, CONTENT_TYPE, HeaderMap, HeaderName, HeaderValue, USER_AGENT,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use url::Url;

use crate::credentials::{load_credentials, save_credentials};
use crate::errors::{GhlError, Result};
use crate::profiles::{Profile, load_profiles};
use crate::redaction::{redact_header_value, redact_json};
use crate::surfaces::Surface;

const DEFAULT_VERSION: &str = "2021-07-28";
const DEFAULT_USER_AGENT: &str =
    "Mozilla/5.0 (compatible; ghl-cli/0.1; +https://github.com/NicholaiVogel/GHL-CLI)";

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct HttpClientConfig {
    pub timeout_seconds: u64,
    pub user_agent: String,
    pub version: String,
}

impl Default for HttpClientConfig {
    fn default() -> Self {
        Self {
            timeout_seconds: 30,
            user_agent: std::env::var("GHL_CLI_USER_AGENT")
                .unwrap_or_else(|_| DEFAULT_USER_AGENT.to_owned()),
            version: DEFAULT_VERSION.to_owned(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RawGetRequest {
    pub surface: Surface,
    pub path: String,
    pub auth_class: AuthClass,
    pub include_body: bool,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum AuthClass {
    Pit,
}

impl std::str::FromStr for AuthClass {
    type Err = GhlError;

    fn from_str(value: &str) -> Result<Self> {
        match value {
            "pit" => Ok(Self::Pit),
            _ => Err(GhlError::Validation {
                message: format!("unsupported auth class `{value}` in this slice; expected pit"),
            }),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RawGetResponse {
    pub method: String,
    pub surface: String,
    pub url: String,
    pub auth_class: String,
    pub status: u16,
    pub success: bool,
    pub headers: BTreeMap<String, String>,
    pub body_included: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub body_json: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub body_text: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PitValidationResult {
    pub profile: String,
    pub location_id: String,
    pub endpoint: String,
    pub status: u16,
    pub success: bool,
    pub credential_ref: String,
    pub warning: Option<String>,
}

pub fn raw_get(
    paths: &crate::ConfigPaths,
    profile_name: Option<&str>,
    request: RawGetRequest,
) -> Result<RawGetResponse> {
    let profiles = load_profiles(paths)?;
    let selected = profiles.selected_name(profile_name).to_owned();
    let profile = profiles.get_required(&selected)?;
    let token = pit_token_for_profile(paths, profile)?;
    let config = HttpClientConfig::default();
    execute_get(profile, &token, request, &config)
}

pub fn validate_pit(
    paths: &crate::ConfigPaths,
    profile_name: Option<&str>,
) -> Result<PitValidationResult> {
    let profiles = load_profiles(paths)?;
    let selected = profiles.selected_name(profile_name).to_owned();
    let profile = profiles.get_required(&selected)?;
    let location_id = profile
        .location_id
        .clone()
        .ok_or_else(|| GhlError::Validation {
            message: format!("profile `{selected}` needs a location id before PIT validation"),
        })?;
    let credential_ref =
        profile
            .credential_refs
            .pit
            .clone()
            .ok_or_else(|| GhlError::CredentialNotFound {
                credential_ref: format!("pit:{selected}"),
            })?;
    let token = pit_token_for_profile(paths, profile)?;
    let endpoint = format!("/locations/{location_id}");
    let config = HttpClientConfig::default();
    let response = execute_get(
        profile,
        &token,
        RawGetRequest {
            surface: Surface::Services,
            path: endpoint.clone(),
            auth_class: AuthClass::Pit,
            include_body: false,
        },
        &config,
    )?;
    if response.success {
        let mut credentials = load_credentials(paths)?;
        credentials.mark_validated(&credential_ref);
        save_credentials(paths, &credentials)?;
    }

    Ok(PitValidationResult {
        profile: selected,
        location_id,
        endpoint,
        status: response.status,
        success: response.success,
        credential_ref,
        warning: if response.success {
            None
        } else {
            Some("PIT validation request completed but did not return a success status.".to_owned())
        },
    })
}

fn execute_get(
    profile: &Profile,
    token: &str,
    request: RawGetRequest,
    config: &HttpClientConfig,
) -> Result<RawGetResponse> {
    let url = build_url(request.surface.base_url(profile), &request.path)?;
    let headers = pit_headers(token, config)?;
    let client = Client::builder()
        .timeout(Duration::from_secs(config.timeout_seconds))
        .build()
        .map_err(|source| GhlError::Network {
            message: source.to_string(),
        })?;
    let response = client
        .get(url.clone())
        .headers(headers)
        .send()
        .map_err(|source| GhlError::Network {
            message: source.to_string(),
        })?;
    let status = response.status();
    let headers = redact_headers(response.headers());
    let text = response.text().map_err(|source| GhlError::Network {
        message: source.to_string(),
    })?;
    let (body_json, body_text) = if request.include_body {
        match serde_json::from_str::<Value>(&text) {
            Ok(value) => (Some(redact_json(&value)), None),
            Err(_) => (None, Some(crate::redaction::redact_token_like(&text))),
        }
    } else {
        (None, None)
    };

    Ok(RawGetResponse {
        method: "GET".to_owned(),
        surface: request.surface.as_str().to_owned(),
        url: redacted_url(&url),
        auth_class: "pit".to_owned(),
        status: status.as_u16(),
        success: status.is_success(),
        headers,
        body_included: request.include_body,
        body_json,
        body_text,
    })
}

fn pit_token_for_profile(paths: &crate::ConfigPaths, profile: &Profile) -> Result<String> {
    let credential_ref =
        profile
            .credential_refs
            .pit
            .as_deref()
            .ok_or_else(|| GhlError::CredentialNotFound {
                credential_ref: format!("pit:{}", profile.name),
            })?;
    let credentials = load_credentials(paths)?;
    let credential =
        credentials
            .get(credential_ref)
            .ok_or_else(|| GhlError::CredentialNotFound {
                credential_ref: credential_ref.to_owned(),
            })?;
    Ok(credential.secret.clone())
}

pub fn build_url(base_url: &str, path: &str) -> Result<Url> {
    if path.starts_with("http://") || path.starts_with("https://") {
        return Err(GhlError::Validation {
            message: "raw request paths must be relative; absolute URLs are refused in this slice"
                .to_owned(),
        });
    }
    if !path.starts_with('/') {
        return Err(GhlError::Validation {
            message: "raw request path must start with `/`".to_owned(),
        });
    }

    let base = Url::parse(base_url).map_err(|source| GhlError::Validation {
        message: format!("invalid base URL `{base_url}`: {source}"),
    })?;
    base.join(path.trim_start_matches('/'))
        .map_err(|source| GhlError::Validation {
            message: format!("invalid request path `{path}`: {source}"),
        })
}

fn pit_headers(token: &str, config: &HttpClientConfig) -> Result<HeaderMap> {
    let mut headers = HeaderMap::new();
    headers.insert(
        AUTHORIZATION,
        HeaderValue::from_str(&format!("Bearer {token}")).map_err(|source| {
            GhlError::Validation {
                message: format!("invalid PIT token header value: {source}"),
            }
        })?,
    );
    headers.insert(ACCEPT, HeaderValue::from_static("application/json"));
    headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
    headers.insert(
        USER_AGENT,
        HeaderValue::from_str(&config.user_agent).map_err(|source| GhlError::Validation {
            message: format!("invalid user-agent header value: {source}"),
        })?,
    );
    headers.insert(
        HeaderName::from_static("version"),
        HeaderValue::from_str(&config.version).map_err(|source| GhlError::Validation {
            message: format!("invalid version header value: {source}"),
        })?,
    );
    Ok(headers)
}

fn redact_headers(headers: &HeaderMap) -> BTreeMap<String, String> {
    headers
        .iter()
        .map(|(name, value)| {
            let value = value.to_str().unwrap_or("<non-utf8>");
            (name.to_string(), redact_header_value(name.as_str(), value))
        })
        .collect()
}

fn redacted_url(url: &Url) -> String {
    let mut url = url.clone();
    if url.query().is_some() {
        let pairs = url
            .query_pairs()
            .map(|(key, value)| (key.to_string(), crate::redaction::redact_token_like(&value)))
            .collect::<Vec<_>>();
        url.set_query(None);
        {
            let mut query = url.query_pairs_mut();
            for (key, value) in pairs {
                query.append_pair(&key, &value);
            }
        }
    }
    url.to_string()
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
    fn build_url_refuses_absolute_paths() {
        let error = build_url("https://services.leadconnectorhq.com", "https://evil.test/")
            .expect_err("absolute URL refused");

        assert_eq!(error.code(), "validation_error");
    }

    #[test]
    fn raw_get_sends_pit_headers_and_redacts_response() {
        let server = MockServer::start();
        let mock = server.mock(|when, then| {
            when.method(GET)
                .path("/locations/loc_123")
                .header("authorization", "Bearer pit-secret")
                .header("version", DEFAULT_VERSION);
            then.status(200)
                .header("content-type", "application/json")
                .json_body(json!({ "id": "loc_123", "token": "secret-token" }));
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

        let response = raw_get(
            &paths,
            Some("default"),
            RawGetRequest {
                surface: Surface::Services,
                path: "/locations/loc_123".to_owned(),
                auth_class: AuthClass::Pit,
                include_body: true,
            },
        )
        .expect("raw get");

        mock.assert();
        assert_eq!(response.status, 200);
        assert_eq!(response.body_json.unwrap()["token"], "[REDACTED]");
    }

    #[test]
    fn pit_validation_uses_location_get_without_body() {
        let server = MockServer::start();
        let mock = server.mock(|when, then| {
            when.method(GET).path("/locations/loc_123");
            then.status(200)
                .json_body(json!({ "id": "loc_123", "name": "Test" }));
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

        let result = validate_pit(&paths, Some("default")).expect("validate");

        mock.assert();
        assert!(result.success);
        assert_eq!(result.endpoint, "/locations/loc_123");
    }
}
