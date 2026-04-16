use serde::{Deserialize, Serialize};
use serde_json::Value;
use url::form_urlencoded::Serializer;

use crate::client::{AuthClass, RawGetRequest, raw_get};
use crate::context::{ResolvedContext, resolve_context, resolve_context_for_dry_run};
use crate::errors::{GhlError, Result};
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
    use httpmock::Method::GET;
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
