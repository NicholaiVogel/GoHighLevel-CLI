use serde::{Deserialize, Serialize};
use serde_json::Value;
use url::form_urlencoded::Serializer;

use crate::client::{AuthClass, RawGetRequest, raw_get};
use crate::context::{ResolvedContext, resolve_context, resolve_context_for_dry_run};
use crate::errors::{GhlError, Result};
use crate::surfaces::Surface;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ConversationStatus {
    All,
    Read,
    Unread,
    Starred,
    Recents,
}

impl ConversationStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::All => "all",
            Self::Read => "read",
            Self::Unread => "unread",
            Self::Starred => "starred",
            Self::Recents => "recents",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ConversationSearchOptions {
    pub contact_id: Option<String>,
    pub query: Option<String>,
    pub status: ConversationStatus,
    pub assigned_to: Option<String>,
    pub limit: u32,
    pub last_message_type: Option<String>,
    pub start_after_date: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ConversationSearchResult {
    pub profile: String,
    pub context: ResolvedContext,
    pub location_id: String,
    pub endpoint: String,
    pub url: String,
    pub status: u16,
    pub success: bool,
    pub search_status: ConversationStatus,
    pub limit: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub body_json: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub body_text: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ConversationSearchDryRun {
    pub method: &'static str,
    pub surface: &'static str,
    pub path: String,
    pub context: ResolvedContext,
    pub location_id: String,
    pub search_status: ConversationStatus,
    pub limit: u32,
    pub auth_class: &'static str,
    pub network: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ConversationGetResult {
    pub profile: String,
    pub context: ResolvedContext,
    pub location_id: String,
    pub conversation_id: String,
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
pub struct ConversationGetDryRun {
    pub method: &'static str,
    pub surface: &'static str,
    pub path: String,
    pub context: ResolvedContext,
    pub location_id: String,
    pub conversation_id: String,
    pub auth_class: &'static str,
    pub network: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ConversationMessagesOptions {
    pub limit: u32,
    pub last_message_id: Option<String>,
    pub message_type: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ConversationMessagesResult {
    pub profile: String,
    pub context: ResolvedContext,
    pub location_id: String,
    pub conversation_id: String,
    pub endpoint: String,
    pub url: String,
    pub status: u16,
    pub success: bool,
    pub limit: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub body_json: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub body_text: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ConversationMessagesDryRun {
    pub method: &'static str,
    pub surface: &'static str,
    pub path: String,
    pub context: ResolvedContext,
    pub location_id: String,
    pub conversation_id: String,
    pub limit: u32,
    pub auth_class: &'static str,
    pub network: bool,
}

pub fn search_conversations(
    paths: &crate::ConfigPaths,
    profile_name: Option<&str>,
    location_override: Option<&str>,
    options: ConversationSearchOptions,
) -> Result<ConversationSearchResult> {
    validate_search_options(&options)?;
    let context = resolve_context(paths, profile_name, location_override, None)?;
    let location_id = context.require_location_id()?.to_owned();
    let endpoint = conversations_search_endpoint(&location_id, &options);
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

    Ok(ConversationSearchResult {
        profile: context.profile.clone(),
        context,
        location_id,
        endpoint,
        url: response.url,
        status: response.status,
        success: response.success,
        search_status: options.status,
        limit: options.limit,
        body_json: response.body_json,
        body_text: response.body_text,
    })
}

pub fn get_conversation(
    paths: &crate::ConfigPaths,
    profile_name: Option<&str>,
    location_override: Option<&str>,
    conversation_id: &str,
) -> Result<ConversationGetResult> {
    validate_conversation_id(conversation_id)?;
    let context = resolve_context(paths, profile_name, location_override, None)?;
    let location_id = context.require_location_id()?.to_owned();
    let endpoint = conversation_endpoint(conversation_id);
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

    Ok(ConversationGetResult {
        profile: context.profile.clone(),
        context,
        location_id,
        conversation_id: conversation_id.to_owned(),
        endpoint,
        url: response.url,
        status: response.status,
        success: response.success,
        body_json: response.body_json,
        body_text: response.body_text,
    })
}

pub fn get_conversation_messages(
    paths: &crate::ConfigPaths,
    profile_name: Option<&str>,
    location_override: Option<&str>,
    conversation_id: &str,
    options: ConversationMessagesOptions,
) -> Result<ConversationMessagesResult> {
    validate_messages_options(conversation_id, &options)?;
    let context = resolve_context(paths, profile_name, location_override, None)?;
    let location_id = context.require_location_id()?.to_owned();
    let endpoint = conversation_messages_endpoint(conversation_id, &options);
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

    Ok(ConversationMessagesResult {
        profile: context.profile.clone(),
        context,
        location_id,
        conversation_id: conversation_id.to_owned(),
        endpoint,
        url: response.url,
        status: response.status,
        success: response.success,
        limit: options.limit,
        body_json: response.body_json,
        body_text: response.body_text,
    })
}

pub fn conversations_search_dry_run(
    paths: &crate::ConfigPaths,
    profile_name: Option<&str>,
    location_override: Option<&str>,
    options: ConversationSearchOptions,
) -> Result<ConversationSearchDryRun> {
    validate_search_options(&options)?;
    let context = resolve_context_for_dry_run(paths, profile_name, location_override, None)?;
    let location_id = context.require_location_id()?.to_owned();

    Ok(ConversationSearchDryRun {
        method: "GET",
        surface: "services",
        path: conversations_search_endpoint(&location_id, &options),
        context,
        location_id,
        search_status: options.status,
        limit: options.limit,
        auth_class: "pit",
        network: false,
    })
}

pub fn get_conversation_dry_run(
    paths: &crate::ConfigPaths,
    profile_name: Option<&str>,
    location_override: Option<&str>,
    conversation_id: &str,
) -> Result<ConversationGetDryRun> {
    validate_conversation_id(conversation_id)?;
    let context = resolve_context_for_dry_run(paths, profile_name, location_override, None)?;
    let location_id = context.require_location_id()?.to_owned();

    Ok(ConversationGetDryRun {
        method: "GET",
        surface: "services",
        path: conversation_endpoint(conversation_id),
        context,
        location_id,
        conversation_id: conversation_id.to_owned(),
        auth_class: "pit",
        network: false,
    })
}

pub fn conversation_messages_dry_run(
    paths: &crate::ConfigPaths,
    profile_name: Option<&str>,
    location_override: Option<&str>,
    conversation_id: &str,
    options: ConversationMessagesOptions,
) -> Result<ConversationMessagesDryRun> {
    validate_messages_options(conversation_id, &options)?;
    let context = resolve_context_for_dry_run(paths, profile_name, location_override, None)?;
    let location_id = context.require_location_id()?.to_owned();

    Ok(ConversationMessagesDryRun {
        method: "GET",
        surface: "services",
        path: conversation_messages_endpoint(conversation_id, &options),
        context,
        location_id,
        conversation_id: conversation_id.to_owned(),
        limit: options.limit,
        auth_class: "pit",
        network: false,
    })
}

fn conversations_search_endpoint(location_id: &str, options: &ConversationSearchOptions) -> String {
    let mut serializer = Serializer::new(String::new());
    serializer.append_pair("locationId", location_id);
    serializer.append_pair("status", options.status.as_str());
    serializer.append_pair("limit", &options.limit.to_string());
    if let Some(contact_id) = trimmed_optional(options.contact_id.clone()) {
        serializer.append_pair("contactId", &contact_id);
    }
    if let Some(query) = trimmed_optional(options.query.clone()) {
        serializer.append_pair("query", &query);
    }
    if let Some(assigned_to) = trimmed_optional(options.assigned_to.clone()) {
        serializer.append_pair("assignedTo", &assigned_to);
    }
    if let Some(last_message_type) = trimmed_optional(options.last_message_type.clone()) {
        serializer.append_pair("lastMessageType", &last_message_type);
    }
    if let Some(start_after_date) = options.start_after_date {
        serializer.append_pair("startAfterDate", &start_after_date.to_string());
    }

    format!("/conversations/search?{}", serializer.finish())
}

fn conversation_endpoint(conversation_id: &str) -> String {
    format!("/conversations/{conversation_id}")
}

fn conversation_messages_endpoint(
    conversation_id: &str,
    options: &ConversationMessagesOptions,
) -> String {
    let mut serializer = Serializer::new(String::new());
    serializer.append_pair("limit", &options.limit.to_string());
    if let Some(last_message_id) = trimmed_optional(options.last_message_id.clone()) {
        serializer.append_pair("lastMessageId", &last_message_id);
    }
    if let Some(message_type) = trimmed_optional(options.message_type.clone()) {
        serializer.append_pair("type", &message_type);
    }

    format!(
        "/conversations/{conversation_id}/messages?{}",
        serializer.finish()
    )
}

fn validate_search_options(options: &ConversationSearchOptions) -> Result<()> {
    validate_limit(options.limit, "conversation search limit")?;
    validate_optional_text(options.contact_id.as_deref(), "conversation contact id")?;
    validate_optional_text(options.query.as_deref(), "conversation query")?;
    validate_optional_text(
        options.assigned_to.as_deref(),
        "conversation assigned-to user id",
    )?;
    validate_optional_text(
        options.last_message_type.as_deref(),
        "conversation last message type",
    )?;

    Ok(())
}

fn validate_messages_options(
    conversation_id: &str,
    options: &ConversationMessagesOptions,
) -> Result<()> {
    validate_conversation_id(conversation_id)?;
    validate_limit(options.limit, "conversation messages limit")?;
    validate_optional_text(options.last_message_id.as_deref(), "last message id")?;
    validate_optional_text(options.message_type.as_deref(), "message type")?;

    Ok(())
}

fn validate_conversation_id(conversation_id: &str) -> Result<()> {
    validate_path_segment(conversation_id, "conversation id")
}

fn validate_limit(limit: u32, label: &str) -> Result<()> {
    if limit == 0 || limit > 100 {
        return Err(GhlError::Validation {
            message: format!("{label} must be between 1 and 100"),
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
    fn conversations_search_uses_location_context_and_redacts_message_bodies() {
        let server = MockServer::start();
        let mock = server.mock(|when, then| {
            when.method(GET)
                .path("/conversations/search")
                .query_param("locationId", "loc_123")
                .query_param("status", "unread")
                .query_param("limit", "20")
                .query_param("contactId", "contact_123")
                .query_param("query", "Sarah")
                .header("authorization", "Bearer pit-secret");
            then.status(200)
                .header("content-type", "application/json")
                .json_body(json!({
                    "conversations": [
                        {
                            "id": "conv_123",
                            "lastMessageBody": "private customer text",
                            "lastMessageType": "TYPE_SMS"
                        }
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
        point_services_base_url(&paths, &server);

        let result = search_conversations(
            &paths,
            Some("default"),
            None,
            ConversationSearchOptions {
                contact_id: Some("contact_123".to_owned()),
                query: Some("Sarah".to_owned()),
                status: ConversationStatus::Unread,
                assigned_to: None,
                limit: 20,
                last_message_type: None,
                start_after_date: None,
            },
        )
        .expect("conversations");

        mock.assert();
        assert_eq!(result.location_id, "loc_123");
        assert_eq!(result.search_status, ConversationStatus::Unread);
        assert_eq!(
            result.body_json.unwrap()["conversations"][0]["lastMessageBody"],
            "[REDACTED]"
        );
    }

    #[test]
    fn conversation_get_requires_location_context() {
        let server = MockServer::start();
        let mock = server.mock(|when, then| {
            when.method(GET)
                .path("/conversations/conv_123")
                .header("authorization", "Bearer pit-secret");
            then.status(200).json_body(json!({
                "id": "conv_123",
                "lastMessageBody": "private customer text"
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
            get_conversation(&paths, Some("default"), None, "conv_123").expect("conversation");

        mock.assert();
        assert_eq!(result.location_id, "loc_123");
        assert_eq!(result.body_json.unwrap()["lastMessageBody"], "[REDACTED]");
    }

    #[test]
    fn conversation_messages_uses_pagination_filters_and_redacts_body() {
        let server = MockServer::start();
        let mock = server.mock(|when, then| {
            when.method(GET)
                .path("/conversations/conv_123/messages")
                .query_param("limit", "10")
                .query_param("lastMessageId", "msg_099")
                .query_param("type", "TYPE_SMS")
                .header("authorization", "Bearer pit-secret");
            then.status(200).json_body(json!({
                "lastMessageId": "msg_100",
                "nextPage": false,
                "messages": [
                    {
                        "id": "msg_100",
                        "messageType": "TYPE_SMS",
                        "body": "private customer text"
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

        let result = get_conversation_messages(
            &paths,
            Some("default"),
            None,
            "conv_123",
            ConversationMessagesOptions {
                limit: 10,
                last_message_id: Some("msg_099".to_owned()),
                message_type: Some("TYPE_SMS".to_owned()),
            },
        )
        .expect("messages");

        mock.assert();
        assert_eq!(result.location_id, "loc_123");
        assert_eq!(
            result.body_json.unwrap()["messages"][0]["body"],
            "[REDACTED]"
        );
    }

    #[test]
    fn conversations_search_dry_run_uses_location_override_without_profile() {
        let temp = tempfile::tempdir().expect("tempdir");
        let paths = resolve_paths(Some(temp.path()));

        let result = conversations_search_dry_run(
            &paths,
            Some("missing"),
            Some("loc_override"),
            ConversationSearchOptions {
                contact_id: None,
                query: None,
                status: ConversationStatus::All,
                assigned_to: None,
                limit: 20,
                last_message_type: Some("TYPE_CALL".to_owned()),
                start_after_date: Some(1_776_300_000_000),
            },
        )
        .expect("dry run");

        assert_eq!(result.method, "GET");
        assert_eq!(result.location_id, "loc_override");
        assert_eq!(
            result.path,
            "/conversations/search?locationId=loc_override&status=all&limit=20&lastMessageType=TYPE_CALL&startAfterDate=1776300000000"
        );
    }

    #[test]
    fn conversation_get_rejects_path_injection() {
        let temp = tempfile::tempdir().expect("tempdir");
        let paths = resolve_paths(Some(temp.path()));

        let error = get_conversation_dry_run(&paths, None, Some("loc_123"), "../conv_123")
            .expect_err("invalid id");

        assert_eq!(error.code(), "validation_error");
    }

    #[test]
    fn conversation_messages_limit_is_bounded() {
        let temp = tempfile::tempdir().expect("tempdir");
        let paths = resolve_paths(Some(temp.path()));

        let error = conversation_messages_dry_run(
            &paths,
            None,
            Some("loc_123"),
            "conv_123",
            ConversationMessagesOptions {
                limit: 0,
                last_message_id: None,
                message_type: None,
            },
        )
        .expect_err("invalid limit");

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
