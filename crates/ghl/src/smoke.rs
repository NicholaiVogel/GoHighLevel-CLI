use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::context::{ResolvedContext, resolve_context, resolve_context_for_dry_run};
use crate::errors::Result;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SmokeRunOptions {
    pub limit: u32,
    pub skip_optional: bool,
    pub contact_query: Option<String>,
    pub contact_email: Option<String>,
    pub contact_phone: Option<String>,
    pub contact_id: Option<String>,
    pub conversation_id: Option<String>,
    pub pipeline_id: Option<String>,
    pub opportunity_id: Option<String>,
}

impl Default for SmokeRunOptions {
    fn default() -> Self {
        Self {
            limit: 1,
            skip_optional: false,
            contact_query: None,
            contact_email: None,
            contact_phone: None,
            contact_id: None,
            conversation_id: None,
            pipeline_id: None,
            opportunity_id: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SmokeRunReport {
    pub ok: bool,
    pub mode: SmokeRunMode,
    pub profile: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub location_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub company_id: Option<String>,
    pub summary: SmokeSummary,
    pub checks: Vec<SmokeCheck>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum SmokeRunMode {
    Live,
    DryRun,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SmokeSummary {
    pub passed: usize,
    pub failed: usize,
    pub skipped: usize,
    pub planned: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SmokeCheck {
    pub name: String,
    pub status: SmokeCheckStatus,
    pub required: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub endpoint_key: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub http_status: Option<u16>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub count: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error_code: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum SmokeCheckStatus {
    Passed,
    Failed,
    Skipped,
    Planned,
}

pub fn smoke_run(
    paths: &crate::ConfigPaths,
    profile_name: Option<&str>,
    location_override: Option<&str>,
    company_override: Option<&str>,
    options: SmokeRunOptions,
) -> SmokeRunReport {
    smoke_run_inner(
        paths,
        profile_name,
        location_override,
        company_override,
        options,
        SmokeRunMode::Live,
    )
}

pub fn smoke_run_dry_run(
    paths: &crate::ConfigPaths,
    profile_name: Option<&str>,
    location_override: Option<&str>,
    company_override: Option<&str>,
    options: SmokeRunOptions,
) -> SmokeRunReport {
    smoke_run_inner(
        paths,
        profile_name,
        location_override,
        company_override,
        options,
        SmokeRunMode::DryRun,
    )
}

fn smoke_run_inner(
    paths: &crate::ConfigPaths,
    profile_name: Option<&str>,
    location_override: Option<&str>,
    company_override: Option<&str>,
    options: SmokeRunOptions,
    mode: SmokeRunMode,
) -> SmokeRunReport {
    let limit = options.limit.clamp(1, 100);
    let (context, context_check) = resolve_smoke_context(
        paths,
        profile_name,
        location_override,
        company_override,
        mode,
    );
    let profile = context
        .as_ref()
        .map(|context| context.profile.clone())
        .or_else(|| profile_name.map(str::to_owned))
        .unwrap_or_else(|| "default".to_owned());
    let location_id = context.as_ref().and_then(|context| {
        context
            .location_id
            .as_ref()
            .map(|value| value.value.clone())
    });
    let company_id = context
        .as_ref()
        .and_then(|context| context.company_id.as_ref().map(|value| value.value.clone()));

    let mut checks = Vec::new();
    checks.push(check_auth_status(paths, profile_name, mode));
    checks.push(context_check);

    let Some(location_id) = location_id.as_deref() else {
        return finish_report(mode, profile, None, company_id, checks);
    };

    if mode == SmokeRunMode::DryRun {
        push_planned_checks(
            &mut checks,
            company_id.as_deref(),
            &options,
            location_id,
            limit,
        );
        return finish_report(
            mode,
            profile,
            Some(location_id.to_owned()),
            company_id,
            checks,
        );
    }

    checks.push(check_locations_get(paths, profile_name, location_id));
    if company_id.is_some() {
        checks.push(check_locations_list(
            paths,
            profile_name,
            location_override,
            company_override,
        ));
    } else {
        checks.push(skipped(
            "locations.list",
            false,
            Some("locations.search"),
            "company context unavailable; pass --company or set a profile company id",
        ));
    }
    checks.push(check_contacts_list(
        paths,
        profile_name,
        location_override,
        limit,
    ));
    checks.push(check_pipelines_list(paths, profile_name, location_override));
    checks.push(check_conversations_search(
        paths,
        profile_name,
        location_override,
        limit,
    ));
    checks.push(check_opportunities_search(
        paths,
        profile_name,
        location_override,
        limit,
    ));

    if !options.skip_optional {
        push_optional_live_checks(
            &mut checks,
            paths,
            profile_name,
            location_override,
            options,
            limit,
        );
    }

    finish_report(
        mode,
        profile,
        Some(location_id.to_owned()),
        company_id,
        checks,
    )
}

fn resolve_smoke_context(
    paths: &crate::ConfigPaths,
    profile_name: Option<&str>,
    location_override: Option<&str>,
    company_override: Option<&str>,
    mode: SmokeRunMode,
) -> (Option<ResolvedContext>, SmokeCheck) {
    let result = if mode == SmokeRunMode::DryRun {
        resolve_context_for_dry_run(paths, profile_name, location_override, company_override)
    } else {
        resolve_context(paths, profile_name, location_override, company_override)
    };

    match result {
        Ok(context) => match context.require_location_id() {
            Ok(_) => (
                Some(context),
                passed("context.location", true, None, None, None),
            ),
            Err(error) => (
                Some(context),
                failed(
                    "context.location",
                    true,
                    None,
                    Some(error.code()),
                    error.to_string(),
                ),
            ),
        },
        Err(error) => (
            None,
            failed(
                "context.location",
                true,
                None,
                Some(error.code()),
                error.to_string(),
            ),
        ),
    }
}

fn check_auth_status(
    paths: &crate::ConfigPaths,
    profile_name: Option<&str>,
    mode: SmokeRunMode,
) -> SmokeCheck {
    if mode == SmokeRunMode::DryRun {
        return match crate::auth_status(paths, profile_name) {
            Ok(status) if status.auth.pit.available => {
                planned("auth.status", true, None, "PIT credential is configured")
            }
            Ok(_) => planned(
                "auth.status",
                true,
                None,
                "would check for a configured PIT credential",
            ),
            Err(_) => planned(
                "auth.status",
                true,
                None,
                "would check the selected profile for a PIT credential",
            ),
        };
    }

    match crate::auth_status(paths, profile_name) {
        Ok(status) if status.auth.pit.available => passed("auth.status", true, None, None, None),
        Ok(_) => failed(
            "auth.status",
            true,
            None,
            Some("validation_error"),
            "PIT credential is not configured for the selected profile",
        ),
        Err(error) => failed(
            "auth.status",
            true,
            None,
            Some(error.code()),
            error.to_string(),
        ),
    }
}

fn push_planned_checks(
    checks: &mut Vec<SmokeCheck>,
    company_id: Option<&str>,
    options: &SmokeRunOptions,
    location_id: &str,
    limit: u32,
) {
    checks.push(planned(
        "locations.get",
        true,
        Some("locations.get"),
        &format!("would GET /locations/{location_id}"),
    ));
    if company_id.is_some() {
        checks.push(planned(
            "locations.list",
            false,
            Some("locations.search"),
            "would GET /locations/search",
        ));
    } else {
        checks.push(skipped(
            "locations.list",
            false,
            Some("locations.search"),
            "company context unavailable; pass --company or set a profile company id",
        ));
    }
    checks.push(planned(
        "contacts.list",
        true,
        Some("contacts.search"),
        &format!("would POST /contacts/search with pageLimit {limit}"),
    ));
    checks.push(planned(
        "pipelines.list",
        true,
        Some("pipelines.list"),
        "would GET /opportunities/pipelines",
    ));
    checks.push(planned(
        "conversations.search",
        true,
        Some("conversations.search"),
        &format!("would GET /conversations/search with limit {limit}"),
    ));
    checks.push(planned(
        "opportunities.search",
        true,
        Some("opportunities.search"),
        &format!("would GET /opportunities/search with limit {limit}"),
    ));

    if options.skip_optional {
        return;
    }

    optional_plan_or_skip(
        checks,
        has_contact_search_input(options),
        "contacts.search",
        Some("contacts.search"),
        "pass --contact-query, --contact-email, or --contact-phone",
    );
    optional_plan_or_skip(
        checks,
        options.contact_id.is_some(),
        "contacts.get",
        Some("contacts.get"),
        "pass --contact-id",
    );
    optional_plan_or_skip(
        checks,
        options.conversation_id.is_some(),
        "conversations.get",
        Some("conversations.get"),
        "pass --conversation-id",
    );
    optional_plan_or_skip(
        checks,
        options.conversation_id.is_some(),
        "conversations.messages",
        Some("conversations.messages"),
        "pass --conversation-id",
    );
    optional_plan_or_skip(
        checks,
        options.pipeline_id.is_some(),
        "pipelines.get",
        Some("pipelines.list"),
        "pass --pipeline-id",
    );
    optional_plan_or_skip(
        checks,
        options.opportunity_id.is_some(),
        "opportunities.get",
        Some("opportunities.get"),
        "pass --opportunity-id",
    );
}

fn push_optional_live_checks(
    checks: &mut Vec<SmokeCheck>,
    paths: &crate::ConfigPaths,
    profile_name: Option<&str>,
    location_override: Option<&str>,
    options: SmokeRunOptions,
    limit: u32,
) {
    if has_contact_search_input(&options) {
        checks.push(check_contacts_search(
            paths,
            profile_name,
            location_override,
            &options,
            limit,
        ));
    } else {
        checks.push(skipped(
            "contacts.search",
            false,
            Some("contacts.search"),
            "pass --contact-query, --contact-email, or --contact-phone",
        ));
    }

    if let Some(contact_id) = options.contact_id.as_deref() {
        checks.push(check_contacts_get(
            paths,
            profile_name,
            location_override,
            contact_id,
        ));
    } else {
        checks.push(skipped(
            "contacts.get",
            false,
            Some("contacts.get"),
            "pass --contact-id",
        ));
    }

    if let Some(conversation_id) = options.conversation_id.as_deref() {
        checks.push(check_conversation_get(
            paths,
            profile_name,
            location_override,
            conversation_id,
        ));
        checks.push(check_conversation_messages(
            paths,
            profile_name,
            location_override,
            conversation_id,
            limit,
        ));
    } else {
        checks.push(skipped(
            "conversations.get",
            false,
            Some("conversations.get"),
            "pass --conversation-id",
        ));
        checks.push(skipped(
            "conversations.messages",
            false,
            Some("conversations.messages"),
            "pass --conversation-id",
        ));
    }

    if let Some(pipeline_id) = options.pipeline_id.as_deref() {
        checks.push(check_pipeline_get(
            paths,
            profile_name,
            location_override,
            pipeline_id,
        ));
    } else {
        checks.push(skipped(
            "pipelines.get",
            false,
            Some("pipelines.list"),
            "pass --pipeline-id",
        ));
    }

    if let Some(opportunity_id) = options.opportunity_id.as_deref() {
        checks.push(check_opportunity_get(
            paths,
            profile_name,
            location_override,
            opportunity_id,
        ));
    } else {
        checks.push(skipped(
            "opportunities.get",
            false,
            Some("opportunities.get"),
            "pass --opportunity-id",
        ));
    }
}

fn check_locations_get(
    paths: &crate::ConfigPaths,
    profile_name: Option<&str>,
    location_id: &str,
) -> SmokeCheck {
    let result = crate::get_location(paths, profile_name, location_id);
    response_check("locations.get", true, Some("locations.get"), result, |_| {
        None
    })
}

fn check_locations_list(
    paths: &crate::ConfigPaths,
    profile_name: Option<&str>,
    location_override: Option<&str>,
    company_override: Option<&str>,
) -> SmokeCheck {
    let options = crate::LocationSearchOptions {
        company_id: company_override.map(str::to_owned),
        email: None,
        skip: 0,
        limit: 1,
        order: crate::LocationSearchOrder::Asc,
    };
    let result = crate::list_locations(paths, profile_name, location_override, options);
    response_check(
        "locations.list",
        false,
        Some("locations.search"),
        result,
        |body| count_array(body, "locations"),
    )
}

fn check_contacts_list(
    paths: &crate::ConfigPaths,
    profile_name: Option<&str>,
    location_override: Option<&str>,
    limit: u32,
) -> SmokeCheck {
    match crate::list_contacts(
        paths,
        profile_name,
        location_override,
        crate::ContactListOptions {
            limit,
            start_after_id: None,
            start_after: None,
        },
    ) {
        Ok(result) if result.success => passed(
            "contacts.list",
            true,
            Some("contacts.search"),
            Some(result.status),
            Some(result.count),
        ),
        Ok(result) => failed(
            "contacts.list",
            true,
            Some("contacts.search"),
            Some("ghl_api_error"),
            "request completed but did not return a success status",
        )
        .with_http_status(result.status)
        .with_count(Some(result.count)),
        Err(error) => failed(
            "contacts.list",
            true,
            Some("contacts.search"),
            Some(error.code()),
            error.to_string(),
        ),
    }
}

fn check_pipelines_list(
    paths: &crate::ConfigPaths,
    profile_name: Option<&str>,
    location_override: Option<&str>,
) -> SmokeCheck {
    let result = crate::list_pipelines(paths, profile_name, location_override);
    response_check(
        "pipelines.list",
        true,
        Some("pipelines.list"),
        result,
        |body| count_array(body, "pipelines"),
    )
}

fn check_conversations_search(
    paths: &crate::ConfigPaths,
    profile_name: Option<&str>,
    location_override: Option<&str>,
    limit: u32,
) -> SmokeCheck {
    let result = crate::search_conversations(
        paths,
        profile_name,
        location_override,
        crate::ConversationSearchOptions {
            contact_id: None,
            query: None,
            status: crate::ConversationStatus::All,
            assigned_to: None,
            limit,
            last_message_type: None,
            start_after_date: None,
        },
    );
    response_check(
        "conversations.search",
        true,
        Some("conversations.search"),
        result,
        |body| count_array(body, "conversations"),
    )
}

fn check_opportunities_search(
    paths: &crate::ConfigPaths,
    profile_name: Option<&str>,
    location_override: Option<&str>,
    limit: u32,
) -> SmokeCheck {
    let result = crate::search_opportunities(
        paths,
        profile_name,
        location_override,
        crate::OpportunitySearchOptions {
            query: None,
            pipeline_id: None,
            pipeline_stage_id: None,
            contact_id: None,
            status: None,
            assigned_to: None,
            limit,
            page: None,
            start_after_id: None,
            start_after: None,
        },
    );
    response_check(
        "opportunities.search",
        true,
        Some("opportunities.search"),
        result,
        |body| count_array(body, "opportunities"),
    )
}

fn check_contacts_search(
    paths: &crate::ConfigPaths,
    profile_name: Option<&str>,
    location_override: Option<&str>,
    options: &SmokeRunOptions,
    limit: u32,
) -> SmokeCheck {
    let result = crate::search_contacts(
        paths,
        profile_name,
        location_override,
        crate::ContactSearchOptions {
            query: options.contact_query.clone(),
            email: options.contact_email.clone(),
            phone: options.contact_phone.clone(),
            limit,
            start_after_id: None,
            start_after: None,
        },
    );
    response_check(
        "contacts.search",
        false,
        Some("contacts.search"),
        result,
        |body| count_array(body, "contacts"),
    )
}

fn check_contacts_get(
    paths: &crate::ConfigPaths,
    profile_name: Option<&str>,
    location_override: Option<&str>,
    contact_id: &str,
) -> SmokeCheck {
    let result = crate::get_contact(paths, profile_name, location_override, contact_id);
    response_check("contacts.get", false, Some("contacts.get"), result, |_| {
        None
    })
}

fn check_conversation_get(
    paths: &crate::ConfigPaths,
    profile_name: Option<&str>,
    location_override: Option<&str>,
    conversation_id: &str,
) -> SmokeCheck {
    let result = crate::get_conversation(paths, profile_name, location_override, conversation_id);
    response_check(
        "conversations.get",
        false,
        Some("conversations.get"),
        result,
        |_| None,
    )
}

fn check_conversation_messages(
    paths: &crate::ConfigPaths,
    profile_name: Option<&str>,
    location_override: Option<&str>,
    conversation_id: &str,
    limit: u32,
) -> SmokeCheck {
    let result = crate::get_conversation_messages(
        paths,
        profile_name,
        location_override,
        conversation_id,
        crate::ConversationMessagesOptions {
            limit,
            last_message_id: None,
            message_type: None,
        },
    );
    response_check(
        "conversations.messages",
        false,
        Some("conversations.messages"),
        result,
        |body| count_array(body, "messages"),
    )
}

fn check_pipeline_get(
    paths: &crate::ConfigPaths,
    profile_name: Option<&str>,
    location_override: Option<&str>,
    pipeline_id: &str,
) -> SmokeCheck {
    match crate::get_pipeline(paths, profile_name, location_override, pipeline_id) {
        Ok(result) if result.success && result.found => passed(
            "pipelines.get",
            false,
            Some("pipelines.list"),
            Some(result.status),
            None,
        ),
        Ok(result) if result.success => failed(
            "pipelines.get",
            false,
            Some("pipelines.list"),
            Some("validation_error"),
            format!("pipeline `{pipeline_id}` was not found in the returned pipeline list"),
        )
        .with_http_status(result.status),
        Ok(result) => failed(
            "pipelines.get",
            false,
            Some("pipelines.list"),
            Some("ghl_api_error"),
            "pipeline list request did not return a success status",
        )
        .with_http_status(result.status),
        Err(error) => failed(
            "pipelines.get",
            false,
            Some("pipelines.list"),
            Some(error.code()),
            error.to_string(),
        ),
    }
}

fn check_opportunity_get(
    paths: &crate::ConfigPaths,
    profile_name: Option<&str>,
    location_override: Option<&str>,
    opportunity_id: &str,
) -> SmokeCheck {
    let result = crate::get_opportunity(paths, profile_name, location_override, opportunity_id);
    response_check(
        "opportunities.get",
        false,
        Some("opportunities.get"),
        result,
        |_| None,
    )
}

trait SmokeResponse {
    fn http_status(&self) -> u16;
    fn success(&self) -> bool;
    fn body_json(&self) -> Option<&Value>;
}

macro_rules! impl_smoke_response {
    ($($ty:ty),+ $(,)?) => {
        $(
            impl SmokeResponse for $ty {
                fn http_status(&self) -> u16 {
                    self.status
                }

                fn success(&self) -> bool {
                    self.success
                }

                fn body_json(&self) -> Option<&Value> {
                    self.body_json.as_ref()
                }
            }
        )+
    };
}

impl_smoke_response!(
    crate::LocationGetResult,
    crate::LocationSearchResult,
    crate::PipelineListResult,
    crate::ConversationSearchResult,
    crate::OpportunitySearchResult,
    crate::ContactSearchResult,
    crate::ContactGetResult,
    crate::ConversationGetResult,
    crate::ConversationMessagesResult,
    crate::OpportunityGetResult,
);

fn response_check<T, F>(
    name: &str,
    required: bool,
    endpoint_key: Option<&str>,
    result: Result<T>,
    count_fn: F,
) -> SmokeCheck
where
    T: SmokeResponse,
    F: FnOnce(&Value) -> Option<usize>,
{
    match result {
        Ok(response) if response.success() => passed(
            name,
            required,
            endpoint_key,
            Some(response.http_status()),
            response.body_json().and_then(count_fn),
        ),
        Ok(response) => failed(
            name,
            required,
            endpoint_key,
            Some("ghl_api_error"),
            "request completed but did not return a success status",
        )
        .with_http_status(response.http_status())
        .with_count(response.body_json().and_then(count_fn)),
        Err(error) => failed(
            name,
            required,
            endpoint_key,
            Some(error.code()),
            error.to_string(),
        ),
    }
}

fn has_contact_search_input(options: &SmokeRunOptions) -> bool {
    options
        .contact_query
        .as_deref()
        .is_some_and(|value| !value.trim().is_empty())
        || options
            .contact_email
            .as_deref()
            .is_some_and(|value| !value.trim().is_empty())
        || options
            .contact_phone
            .as_deref()
            .is_some_and(|value| !value.trim().is_empty())
}

fn count_array(body: &Value, field: &str) -> Option<usize> {
    body.get(field).and_then(Value::as_array).map(Vec::len)
}

fn optional_plan_or_skip(
    checks: &mut Vec<SmokeCheck>,
    should_plan: bool,
    name: &str,
    endpoint_key: Option<&str>,
    skip_message: &str,
) {
    if should_plan {
        checks.push(planned(
            name,
            false,
            endpoint_key,
            "would execute optional read check",
        ));
    } else {
        checks.push(skipped(name, false, endpoint_key, skip_message));
    }
}

fn finish_report(
    mode: SmokeRunMode,
    profile: String,
    location_id: Option<String>,
    company_id: Option<String>,
    checks: Vec<SmokeCheck>,
) -> SmokeRunReport {
    let summary = summarize(&checks);
    let ok = summary.failed == 0;

    SmokeRunReport {
        ok,
        mode,
        profile,
        location_id,
        company_id,
        summary,
        checks,
    }
}

fn summarize(checks: &[SmokeCheck]) -> SmokeSummary {
    SmokeSummary {
        passed: checks
            .iter()
            .filter(|check| check.status == SmokeCheckStatus::Passed)
            .count(),
        failed: checks
            .iter()
            .filter(|check| check.status == SmokeCheckStatus::Failed)
            .count(),
        skipped: checks
            .iter()
            .filter(|check| check.status == SmokeCheckStatus::Skipped)
            .count(),
        planned: checks
            .iter()
            .filter(|check| check.status == SmokeCheckStatus::Planned)
            .count(),
    }
}

fn passed(
    name: &str,
    required: bool,
    endpoint_key: Option<&str>,
    http_status: Option<u16>,
    count: Option<usize>,
) -> SmokeCheck {
    SmokeCheck {
        name: name.to_owned(),
        status: SmokeCheckStatus::Passed,
        required,
        endpoint_key: endpoint_key.map(str::to_owned),
        http_status,
        count,
        error_code: None,
        message: None,
    }
}

fn planned(name: &str, required: bool, endpoint_key: Option<&str>, message: &str) -> SmokeCheck {
    SmokeCheck {
        name: name.to_owned(),
        status: SmokeCheckStatus::Planned,
        required,
        endpoint_key: endpoint_key.map(str::to_owned),
        http_status: None,
        count: None,
        error_code: None,
        message: Some(message.to_owned()),
    }
}

fn skipped(name: &str, required: bool, endpoint_key: Option<&str>, message: &str) -> SmokeCheck {
    SmokeCheck {
        name: name.to_owned(),
        status: SmokeCheckStatus::Skipped,
        required,
        endpoint_key: endpoint_key.map(str::to_owned),
        http_status: None,
        count: None,
        error_code: None,
        message: Some(message.to_owned()),
    }
}

fn failed(
    name: &str,
    required: bool,
    endpoint_key: Option<&str>,
    error_code: Option<&str>,
    message: impl Into<String>,
) -> SmokeCheck {
    SmokeCheck {
        name: name.to_owned(),
        status: SmokeCheckStatus::Failed,
        required,
        endpoint_key: endpoint_key.map(str::to_owned),
        http_status: None,
        count: None,
        error_code: error_code.map(str::to_owned),
        message: Some(message.into()),
    }
}

impl SmokeCheck {
    fn with_http_status(mut self, status: u16) -> Self {
        self.http_status = Some(status);
        self
    }

    fn with_count(mut self, count: Option<usize>) -> Self {
        self.count = count;
        self
    }
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
    fn smoke_run_dry_run_plans_required_checks_without_credentials_when_location_override_exists() {
        let temp = tempfile::tempdir().expect("tempdir");
        let paths = resolve_paths(Some(temp.path()));

        let report = smoke_run_dry_run(
            &paths,
            Some("missing"),
            Some("loc_123"),
            None,
            SmokeRunOptions::default(),
        );

        assert!(report.ok);
        assert_eq!(report.mode, SmokeRunMode::DryRun);
        assert_eq!(report.location_id.as_deref(), Some("loc_123"));
        assert!(report.checks.iter().any(
            |check| check.name == "locations.get" && check.status == SmokeCheckStatus::Planned
        ));
    }

    #[test]
    fn smoke_run_executes_required_read_checks_and_skips_missing_optional_inputs() {
        let server = MockServer::start();
        server.mock(|when, then| {
            when.method(GET).path("/locations/loc_123");
            then.status(200).json_body(json!({ "id": "loc_123" }));
        });
        server.mock(|when, then| {
            when.method(POST).path("/contacts/search").json_body(json!({
                "locationId": "loc_123",
                "pageLimit": 1
            }));
            then.status(200).json_body(json!({
                "contacts": [{ "id": "contact_123" }],
                "total": 1
            }));
        });
        server.mock(|when, then| {
            when.method(GET)
                .path("/opportunities/pipelines")
                .query_param("locationId", "loc_123");
            then.status(200).json_body(json!({
                "pipelines": [{ "id": "pipe_123", "name": "Sales", "stages": [] }]
            }));
        });
        server.mock(|when, then| {
            when.method(GET)
                .path("/conversations/search")
                .query_param("locationId", "loc_123")
                .query_param("status", "all")
                .query_param("limit", "1");
            then.status(200).json_body(json!({
                "conversations": [],
                "total": 0
            }));
        });
        server.mock(|when, then| {
            when.method(GET)
                .path("/opportunities/search")
                .query_param("location_id", "loc_123")
                .query_param("limit", "1");
            then.status(200).json_body(json!({
                "opportunities": [],
                "meta": { "total": 0 }
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

        let report = smoke_run(
            &paths,
            Some("default"),
            None,
            None,
            SmokeRunOptions::default(),
        );

        assert!(report.ok, "{report:#?}");
        assert_eq!(report.summary.failed, 0);
        assert!(
            report
                .checks
                .iter()
                .any(|check| check.name == "contacts.search"
                    && check.status == SmokeCheckStatus::Skipped)
        );
    }

    #[test]
    fn smoke_run_reports_failed_required_check() {
        let server = MockServer::start();
        server.mock(|when, then| {
            when.method(GET).path("/locations/loc_123");
            then.status(500)
                .json_body(json!({ "message": "upstream error" }));
        });
        server.mock(|when, then| {
            when.method(POST).path("/contacts/search");
            then.status(200)
                .json_body(json!({ "contacts": [], "total": 0 }));
        });
        server.mock(|when, then| {
            when.method(GET)
                .path("/opportunities/pipelines")
                .query_param("locationId", "loc_123");
            then.status(200).json_body(json!({ "pipelines": [] }));
        });
        server.mock(|when, then| {
            when.method(GET).path("/conversations/search");
            then.status(200)
                .json_body(json!({ "conversations": [], "total": 0 }));
        });
        server.mock(|when, then| {
            when.method(GET).path("/opportunities/search");
            then.status(200)
                .json_body(json!({ "opportunities": [], "meta": { "total": 0 } }));
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

        let report = smoke_run(
            &paths,
            Some("default"),
            None,
            None,
            SmokeRunOptions {
                skip_optional: true,
                ..SmokeRunOptions::default()
            },
        );

        assert!(!report.ok);
        assert!(
            report
                .checks
                .iter()
                .any(|check| check.name == "locations.get"
                    && check.status == SmokeCheckStatus::Failed
                    && check.http_status == Some(500))
        );
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
