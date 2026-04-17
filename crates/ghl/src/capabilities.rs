use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::auth::{AuthStatus, auth_status};
use crate::errors::Result;
use crate::metadata::{CommandMetadata, command_by_key, implemented_commands};
use crate::profiles::{ProfilePolicy, load_profiles};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CapabilityReport {
    pub schema_version: u32,
    pub profile: String,
    pub profile_exists: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub location_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub company_id: Option<String>,
    pub auth_classes: Vec<String>,
    pub capabilities: BTreeMap<String, CapabilityCheck>,
    pub warnings: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CapabilityCheck {
    pub capability: String,
    pub state: CapabilityState,
    pub confidence: CapabilityConfidence,
    pub implemented: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub command_key: Option<String>,
    pub auth_classes: Vec<String>,
    pub endpoint_keys: Vec<String>,
    pub policy_flags: Vec<String>,
    pub reasons: Vec<String>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum CapabilityState {
    Available,
    ExpectedAvailable,
    RequiresPit,
    RequiresLocationContext,
    RequiresCompanyContext,
    BlockedByPolicy,
    NotImplemented,
    Unknown,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum CapabilityConfidence {
    Known,
    Inferred,
    Unknown,
}

#[derive(Debug, Clone)]
struct CapabilityContext {
    profile: String,
    profile_exists: bool,
    location_id: Option<String>,
    company_id: Option<String>,
    pit_available: bool,
    auth_classes: Vec<String>,
    policy: Option<ProfilePolicy>,
    warnings: Vec<String>,
}

pub fn capability_report(
    paths: &crate::ConfigPaths,
    profile_name: Option<&str>,
    location_override: Option<&str>,
    company_override: Option<&str>,
) -> Result<CapabilityReport> {
    let context = capability_context(paths, profile_name, location_override, company_override)?;
    let capabilities = implemented_commands()
        .into_iter()
        .map(|command| {
            let check = evaluate_command(&context, command);
            (check.capability.clone(), check)
        })
        .collect();

    Ok(CapabilityReport {
        schema_version: 1,
        profile: context.profile,
        profile_exists: context.profile_exists,
        location_id: context.location_id,
        company_id: context.company_id,
        auth_classes: context.auth_classes,
        capabilities,
        warnings: context.warnings,
    })
}

pub fn check_capability(
    paths: &crate::ConfigPaths,
    profile_name: Option<&str>,
    location_override: Option<&str>,
    company_override: Option<&str>,
    capability: &str,
) -> Result<CapabilityCheck> {
    let context = capability_context(paths, profile_name, location_override, company_override)?;
    if let Some(command) = command_by_key(capability) {
        return Ok(evaluate_command(&context, command));
    }

    Ok(evaluate_planned_capability(&context, capability))
}

fn capability_context(
    paths: &crate::ConfigPaths,
    profile_name: Option<&str>,
    location_override: Option<&str>,
    company_override: Option<&str>,
) -> Result<CapabilityContext> {
    let profiles = load_profiles(paths)?;
    let selected = profiles.selected_name(profile_name).to_owned();
    let profile = profiles.profiles.get(&selected);
    let auth = auth_status(paths, Some(&selected)).ok();
    let mut warnings = Vec::new();
    if profile.is_none() {
        warnings.push(format!(
            "profile `{selected}` is not configured; local-only capabilities remain available"
        ));
    }
    if auth.is_none() {
        warnings.push("auth status is unavailable for the selected profile".to_owned());
    }

    let location_id = location_override
        .filter(|value| !value.trim().is_empty())
        .map(str::to_owned)
        .or_else(|| profile.and_then(|profile| profile.location_id.clone()));
    let company_id = company_override
        .filter(|value| !value.trim().is_empty())
        .map(str::to_owned)
        .or_else(|| profile.and_then(|profile| profile.company_id.clone()));
    let auth_classes = available_auth_classes(auth.as_ref());
    let pit_available = auth.as_ref().is_some_and(|auth| auth.auth.pit.available);

    Ok(CapabilityContext {
        profile: selected,
        profile_exists: profile.is_some(),
        location_id,
        company_id,
        pit_available,
        auth_classes,
        policy: profile.map(|profile| profile.policy.clone()),
        warnings,
    })
}

fn available_auth_classes(auth: Option<&AuthStatus>) -> Vec<String> {
    let Some(auth) = auth else {
        return Vec::new();
    };
    let mut classes = Vec::new();
    if auth.auth.pit.available {
        classes.push("pit".to_owned());
    }
    if auth.auth.session.available {
        classes.push("session".to_owned());
    }
    if auth.auth.firebase.available {
        classes.push("firebase".to_owned());
    }
    classes
}

fn evaluate_command(context: &CapabilityContext, command: CommandMetadata) -> CapabilityCheck {
    let mut reasons = Vec::new();
    if command.offline {
        reasons.push("command is local-only and does not require GHL network access".to_owned());
        return CapabilityCheck {
            capability: command.command_key.clone(),
            state: CapabilityState::Available,
            confidence: CapabilityConfidence::Known,
            implemented: command.implemented,
            command_key: Some(command.command_key),
            auth_classes: command.auth_classes,
            endpoint_keys: command.endpoint_keys,
            policy_flags: command.policy_flags,
            reasons,
        };
    }

    if command.auth_classes.iter().any(|class| class == "pit") && !context.pit_available {
        reasons
            .push("selected profile does not have a locally available PIT credential".to_owned());
        return check_from_command(
            command,
            CapabilityState::RequiresPit,
            CapabilityConfidence::Known,
            reasons,
        );
    }

    if requires_company_context(&command.command_key) && context.company_id.is_none() {
        reasons.push(
            "command requires company context; pass --company or store a profile company id"
                .to_owned(),
        );
        return check_from_command(
            command,
            CapabilityState::RequiresCompanyContext,
            CapabilityConfidence::Known,
            reasons,
        );
    }

    if requires_location_context(&command.command_key) && context.location_id.is_none() {
        reasons.push(
            "command requires location context; pass --location or store a profile location id"
                .to_owned(),
        );
        return check_from_command(
            command,
            CapabilityState::RequiresLocationContext,
            CapabilityConfidence::Known,
            reasons,
        );
    }

    reasons.push(
        "local auth and required context are present; no live permission probe was run".to_owned(),
    );
    check_from_command(
        command,
        CapabilityState::ExpectedAvailable,
        CapabilityConfidence::Inferred,
        reasons,
    )
}

fn check_from_command(
    command: CommandMetadata,
    state: CapabilityState,
    confidence: CapabilityConfidence,
    reasons: Vec<String>,
) -> CapabilityCheck {
    CapabilityCheck {
        capability: command.command_key.clone(),
        state,
        confidence,
        implemented: command.implemented,
        command_key: Some(command.command_key),
        auth_classes: command.auth_classes,
        endpoint_keys: command.endpoint_keys,
        policy_flags: command.policy_flags,
        reasons,
    }
}

fn evaluate_planned_capability(context: &CapabilityContext, capability: &str) -> CapabilityCheck {
    let lower = capability.to_ascii_lowercase();
    let mut reasons = Vec::new();
    let mut state = CapabilityState::NotImplemented;
    let mut confidence = CapabilityConfidence::Known;
    let mut policy_flags = Vec::new();

    if is_messaging_like(&lower) {
        policy_flags.push("allow_messaging".to_owned());
        if context
            .policy
            .as_ref()
            .is_none_or(|policy| !policy.allow_messaging)
        {
            state = CapabilityState::BlockedByPolicy;
            reasons.push("profile policy blocks messaging actions by default".to_owned());
        }
    } else if is_payment_like(&lower) {
        policy_flags.push("allow_payment_actions".to_owned());
        if context
            .policy
            .as_ref()
            .is_none_or(|policy| !policy.allow_payment_actions)
        {
            state = CapabilityState::BlockedByPolicy;
            reasons.push("profile policy blocks payment actions by default".to_owned());
        }
    } else if is_destructive_like(&lower) {
        policy_flags.push("allow_destructive".to_owned());
        if context
            .policy
            .as_ref()
            .is_none_or(|policy| !policy.allow_destructive)
        {
            state = CapabilityState::BlockedByPolicy;
            reasons.push("profile policy blocks write/destructive actions by default".to_owned());
        }
    }

    if reasons.is_empty() {
        reasons.push("capability is not implemented in the local command schema yet".to_owned());
        if !context.profile_exists {
            confidence = CapabilityConfidence::Unknown;
            state = CapabilityState::Unknown;
            reasons.push(
                "selected profile is unavailable, so local policy could not be inspected"
                    .to_owned(),
            );
        }
    }

    CapabilityCheck {
        capability: capability.to_owned(),
        state,
        confidence,
        implemented: false,
        command_key: None,
        auth_classes: Vec::new(),
        endpoint_keys: Vec::new(),
        policy_flags,
        reasons,
    }
}

fn requires_company_context(command_key: &str) -> bool {
    matches!(command_key, "locations.list" | "locations.search")
}

fn requires_location_context(command_key: &str) -> bool {
    matches!(
        command_key,
        "auth.pit.validate"
            | "contacts.list"
            | "contacts.search"
            | "contacts.get"
            | "conversations.search"
            | "conversations.get"
            | "conversations.messages"
            | "pipelines.list"
            | "pipelines.get"
            | "opportunities.search"
            | "opportunities.get"
            | "calendars.list"
            | "calendars.get"
            | "calendars.events"
            | "calendars.free_slots"
            | "users.list"
            | "users.get"
            | "teams.list"
            | "smoke.run"
    )
}

fn is_destructive_like(value: &str) -> bool {
    value.contains(".write")
        || value.contains(".create")
        || value.contains(".update")
        || value.contains(".delete")
        || value.contains(".cancel")
        || value.contains(".complete")
        || value.contains(".bulk")
}

fn is_messaging_like(value: &str) -> bool {
    value.contains("message")
        || value.contains("send_sms")
        || value.contains("send_email")
        || value.contains("sms")
        || value.contains("email.send")
}

fn is_payment_like(value: &str) -> bool {
    value.contains("payment") || value.contains("invoice") || value.contains("order")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::auth::add_pit;
    use crate::config::resolve_paths;

    #[test]
    fn implemented_local_command_is_available_without_profile() {
        let temp = tempfile::tempdir().expect("tempdir");
        let paths = resolve_paths(Some(temp.path()));

        let check = check_capability(&paths, None, None, None, "commands.schema").expect("check");

        assert_eq!(check.state, CapabilityState::Available);
        assert_eq!(check.confidence, CapabilityConfidence::Known);
    }

    #[test]
    fn remote_command_requires_pit_when_profile_missing() {
        let temp = tempfile::tempdir().expect("tempdir");
        let paths = resolve_paths(Some(temp.path()));

        let check = check_capability(&paths, None, None, None, "contacts.list").expect("check");

        assert_eq!(check.state, CapabilityState::RequiresPit);
    }

    #[test]
    fn remote_command_requires_location_when_pit_exists_without_location() {
        let temp = tempfile::tempdir().expect("tempdir");
        let paths = resolve_paths(Some(temp.path()));
        add_pit(&paths, "default", "pit-secret".to_owned(), None, None, true).expect("pit");

        let check = check_capability(&paths, None, None, None, "contacts.list").expect("check");

        assert_eq!(check.state, CapabilityState::RequiresLocationContext);
    }

    #[test]
    fn remote_command_is_expected_available_with_auth_and_context() {
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

        let check = check_capability(&paths, None, None, None, "contacts.list").expect("check");

        assert_eq!(check.state, CapabilityState::ExpectedAvailable);
        assert_eq!(check.confidence, CapabilityConfidence::Inferred);
    }

    #[test]
    fn planned_write_is_blocked_by_default_policy() {
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

        let check = check_capability(&paths, None, None, None, "contacts.write").expect("check");

        assert_eq!(check.state, CapabilityState::BlockedByPolicy);
        assert_eq!(check.policy_flags, vec!["allow_destructive".to_owned()]);
    }
}
