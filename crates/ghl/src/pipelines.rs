use serde::{Deserialize, Serialize};
use serde_json::Value;
use url::form_urlencoded::Serializer;

use crate::client::{AuthClass, RawGetRequest, raw_get};
use crate::context::{ResolvedContext, resolve_context, resolve_context_for_dry_run};
use crate::errors::{GhlError, Result};
use crate::surfaces::Surface;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PipelineListResult {
    pub profile: String,
    pub context: ResolvedContext,
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
pub struct PipelineListDryRun {
    pub method: &'static str,
    pub surface: &'static str,
    pub path: String,
    pub context: ResolvedContext,
    pub location_id: String,
    pub auth_class: &'static str,
    pub network: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PipelineGetResult {
    pub profile: String,
    pub context: ResolvedContext,
    pub location_id: String,
    pub pipeline_id: String,
    pub endpoint: String,
    pub url: String,
    pub status: u16,
    pub success: bool,
    pub found: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pipeline_json: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub body_json: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub body_text: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PipelineGetDryRun {
    pub method: &'static str,
    pub surface: &'static str,
    pub path: String,
    pub context: ResolvedContext,
    pub location_id: String,
    pub pipeline_id: String,
    pub note: &'static str,
    pub auth_class: &'static str,
    pub network: bool,
}

pub fn list_pipelines(
    paths: &crate::ConfigPaths,
    profile_name: Option<&str>,
    location_override: Option<&str>,
) -> Result<PipelineListResult> {
    let context = resolve_context(paths, profile_name, location_override, None)?;
    let location_id = context.require_location_id()?.to_owned();
    let endpoint = pipelines_endpoint(&location_id);
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

    Ok(PipelineListResult {
        profile: context.profile.clone(),
        context,
        location_id,
        endpoint,
        url: response.url,
        status: response.status,
        success: response.success,
        body_json: response.body_json,
        body_text: response.body_text,
    })
}

pub fn get_pipeline(
    paths: &crate::ConfigPaths,
    profile_name: Option<&str>,
    location_override: Option<&str>,
    pipeline_id: &str,
) -> Result<PipelineGetResult> {
    validate_pipeline_id(pipeline_id)?;
    let list = list_pipelines(paths, profile_name, location_override)?;
    let pipeline_json = list
        .body_json
        .as_ref()
        .and_then(|body| find_pipeline(body, pipeline_id));

    Ok(PipelineGetResult {
        profile: list.profile,
        context: list.context,
        location_id: list.location_id,
        pipeline_id: pipeline_id.to_owned(),
        endpoint: list.endpoint,
        url: list.url,
        status: list.status,
        success: list.success,
        found: pipeline_json.is_some(),
        pipeline_json,
        body_json: list.body_json,
        body_text: list.body_text,
    })
}

pub fn pipelines_list_dry_run(
    paths: &crate::ConfigPaths,
    profile_name: Option<&str>,
    location_override: Option<&str>,
) -> Result<PipelineListDryRun> {
    let context = resolve_context_for_dry_run(paths, profile_name, location_override, None)?;
    let location_id = context.require_location_id()?.to_owned();

    Ok(PipelineListDryRun {
        method: "GET",
        surface: "services",
        path: pipelines_endpoint(&location_id),
        context,
        location_id,
        auth_class: "pit",
        network: false,
    })
}

pub fn get_pipeline_dry_run(
    paths: &crate::ConfigPaths,
    profile_name: Option<&str>,
    location_override: Option<&str>,
    pipeline_id: &str,
) -> Result<PipelineGetDryRun> {
    validate_pipeline_id(pipeline_id)?;
    let context = resolve_context_for_dry_run(paths, profile_name, location_override, None)?;
    let location_id = context.require_location_id()?.to_owned();

    Ok(PipelineGetDryRun {
        method: "GET",
        surface: "services",
        path: pipelines_endpoint(&location_id),
        context,
        location_id,
        pipeline_id: pipeline_id.to_owned(),
        note: "GHL exposes pipeline read through list; this command filters by pipeline id client-side.",
        auth_class: "pit",
        network: false,
    })
}

fn pipelines_endpoint(location_id: &str) -> String {
    let mut serializer = Serializer::new(String::new());
    serializer.append_pair("locationId", location_id);
    format!("/opportunities/pipelines?{}", serializer.finish())
}

fn find_pipeline(body: &Value, pipeline_id: &str) -> Option<Value> {
    body.get("pipelines")
        .and_then(Value::as_array)?
        .iter()
        .find(|pipeline| pipeline.get("id").and_then(Value::as_str) == Some(pipeline_id))
        .cloned()
}

fn validate_pipeline_id(pipeline_id: &str) -> Result<()> {
    validate_path_segment(pipeline_id, "pipeline id")
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

#[cfg(test)]
mod tests {
    use httpmock::Method::GET;
    use httpmock::MockServer;
    use serde_json::json;

    use super::*;
    use crate::auth::add_pit;
    use crate::config::resolve_paths;

    #[test]
    fn pipelines_list_uses_location_context() {
        let server = MockServer::start();
        let mock = server.mock(|when, then| {
            when.method(GET)
                .path("/opportunities/pipelines")
                .query_param("locationId", "loc_123")
                .header("authorization", "Bearer pit-secret");
            then.status(200).json_body(json!({
                "pipelines": [
                    {
                        "id": "pipe_123",
                        "name": "Sales",
                        "locationId": "loc_123",
                        "stages": []
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
        point_services_base_url(&paths, &server);

        let result = list_pipelines(&paths, Some("default"), None).expect("pipelines");

        mock.assert();
        assert_eq!(result.location_id, "loc_123");
        assert_eq!(result.body_json.unwrap()["pipelines"][0]["id"], "pipe_123");
    }

    #[test]
    fn pipelines_get_filters_list_response_client_side() {
        let server = MockServer::start();
        let mock = server.mock(|when, then| {
            when.method(GET)
                .path("/opportunities/pipelines")
                .query_param("locationId", "loc_123");
            then.status(200).json_body(json!({
                "pipelines": [
                    { "id": "pipe_a", "name": "A", "stages": [] },
                    { "id": "pipe_b", "name": "B", "stages": [] }
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

        let result = get_pipeline(&paths, Some("default"), None, "pipe_b").expect("pipeline");

        mock.assert();
        assert!(result.found);
        assert_eq!(result.pipeline_json.unwrap()["name"], "B");
    }

    #[test]
    fn pipelines_list_dry_run_uses_location_override_without_profile() {
        let temp = tempfile::tempdir().expect("tempdir");
        let paths = resolve_paths(Some(temp.path()));

        let result =
            pipelines_list_dry_run(&paths, Some("missing"), Some("loc_override")).expect("dry");

        assert_eq!(
            result.path,
            "/opportunities/pipelines?locationId=loc_override"
        );
        assert_eq!(result.location_id, "loc_override");
    }

    #[test]
    fn pipeline_get_rejects_path_injection() {
        let temp = tempfile::tempdir().expect("tempdir");
        let paths = resolve_paths(Some(temp.path()));

        let error = get_pipeline_dry_run(&paths, None, Some("loc_123"), "../pipe_123")
            .expect_err("invalid id");

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
