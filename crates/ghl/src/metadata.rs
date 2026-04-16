use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CommandSchema {
    pub schema_version: u32,
    pub default_format: String,
    pub error_shape: String,
    pub commands: Vec<CommandMetadata>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CommandMetadata {
    pub command_key: String,
    pub path: String,
    pub summary: String,
    pub phase: String,
    pub stability: String,
    pub implemented: bool,
    pub auth_classes: Vec<String>,
    pub policy_flags: Vec<String>,
    pub endpoint_keys: Vec<String>,
    pub output_schema: String,
    pub offline: bool,
}

pub fn command_schema() -> CommandSchema {
    CommandSchema {
        schema_version: 1,
        default_format: "json".to_owned(),
        error_shape: "{ ok: false, error: { code, message, exit_code, details, hint? }, meta }"
            .to_owned(),
        commands: implemented_commands(),
    }
}

pub fn implemented_commands() -> Vec<CommandMetadata> {
    vec![
        local(
            "commands.schema",
            "commands schema",
            "Print machine-readable command metadata.",
            "CommandSchema",
            "0",
        ),
        local(
            "config.path",
            "config path",
            "Print resolved config, data, cache, audit, job, and lock paths.",
            "ConfigPaths",
            "0",
        ),
        local(
            "config.show",
            "config show",
            "Print redacted local CLI configuration and profile names.",
            "CliConfig",
            "1",
        ),
        local(
            "config.doctor",
            "config doctor",
            "Check local config paths without creating or mutating files.",
            "ConfigDoctor",
            "0",
        ),
        local(
            "auth.pit.add",
            "auth pit add",
            "Store a local PIT credential reference for a profile without printing the token.",
            "PitAddResult",
            "1",
        ),
        remote_pit(
            "auth.pit.validate",
            "auth pit validate",
            "Validate the local PIT with a low-risk GET /locations/{location_id} request without printing the body.",
            "PitValidationResult",
            "1",
            &["locations.get"],
        ),
        local(
            "auth.pit.list_local",
            "auth pit list-local",
            "List locally stored PIT credential references with redacted previews.",
            "LocalPitList",
            "1",
        ),
        local(
            "auth.pit.remove_local",
            "auth pit remove-local <credential-ref>",
            "Remove one local PIT credential reference and clear profile links.",
            "PitRemoveResult",
            "1",
        ),
        local(
            "auth.status",
            "auth status",
            "Report available local auth classes for a profile.",
            "AuthStatus",
            "1",
        ),
        local(
            "profiles.list",
            "profiles list",
            "List configured local profiles.",
            "ProfileList",
            "1",
        ),
        local(
            "profiles.show",
            "profiles show <name>",
            "Show one redacted local profile.",
            "Profile",
            "1",
        ),
        local(
            "profiles.set_default",
            "profiles set-default <name>",
            "Set the default local profile.",
            "ProfileDefaultResult",
            "1",
        ),
        local(
            "profiles.set_default_company",
            "profiles set-default-company <name> <company-id>",
            "Set the default GHL company id for a profile.",
            "ProfileCompanyResult",
            "2",
        ),
        local(
            "profiles.set_default_location",
            "profiles set-default-location <name> <location-id>",
            "Set the default GHL location id for a profile.",
            "ProfileLocationResult",
            "1",
        ),
        local(
            "profiles.policy.show",
            "profiles policy show <name>",
            "Show profile safety policy.",
            "ProfilePolicy",
            "1",
        ),
        local(
            "profiles.policy.set",
            "profiles policy set <name>",
            "Update profile safety policy flags.",
            "ProfilePolicy",
            "1",
        ),
        local(
            "profiles.policy.reset",
            "profiles policy reset <name> --yes",
            "Reset profile safety policy to safe defaults.",
            "ProfilePolicy",
            "1",
        ),
        local(
            "errors.list",
            "errors list",
            "List the standard error-code registry.",
            "ErrorDefinitionList",
            "0",
        ),
        local(
            "errors.show",
            "errors show <error-code>",
            "Show one standard error-code definition.",
            "ErrorDefinition",
            "0",
        ),
        local(
            "endpoints.list",
            "endpoints list",
            "List bundled endpoint manifest entries.",
            "EndpointList",
            "0",
        ),
        local(
            "endpoints.show",
            "endpoints show <endpoint-key>",
            "Show one bundled endpoint manifest entry.",
            "EndpointDefinition",
            "0",
        ),
        local(
            "endpoints.coverage",
            "endpoints coverage",
            "Summarize endpoint manifest coverage.",
            "EndpointCoverage",
            "0",
        ),
        remote_pit(
            "raw.request",
            "raw request",
            "Execute a guarded read-only raw GET request with PIT auth, or preview it with --dry-run.",
            "RawGetResponse",
            "1",
            &[],
        ),
        remote_pit(
            "locations.get",
            "locations get <location-id>",
            "Fetch one location by id with PIT auth and redacted response output.",
            "LocationGetResult",
            "2",
            &["locations.get"],
        ),
        remote_pit(
            "locations.list",
            "locations list [--company <company-id>]",
            "List locations for a company using GHL's /locations/search endpoint.",
            "LocationSearchResult",
            "2",
            &["locations.search"],
        ),
        remote_pit(
            "locations.search",
            "locations search <query> [--company <company-id>]",
            "Search locations using GHL's current email filter.",
            "LocationSearchResult",
            "2",
            &["locations.search"],
        ),
        remote_pit(
            "contacts.search",
            "contacts search [<query>] [--email <email>] [--phone <phone>] [--limit <n>]",
            "Search contacts in the resolved location using query and exact email or phone filters.",
            "ContactSearchResult",
            "2",
            &["contacts.search"],
        ),
        remote_pit(
            "contacts.get",
            "contacts get <contact-id>",
            "Fetch one contact by id within the resolved location context.",
            "ContactGetResult",
            "2",
            &["contacts.get"],
        ),
        remote_pit(
            "conversations.search",
            "conversations search [--contact <contact-id>] [--query <query>] [--status <status>] [--limit <n>]",
            "Search conversations in the resolved location.",
            "ConversationSearchResult",
            "2",
            &["conversations.search"],
        ),
        remote_pit(
            "conversations.get",
            "conversations get <conversation-id>",
            "Fetch one conversation by id within the resolved location context.",
            "ConversationGetResult",
            "2",
            &["conversations.get"],
        ),
        remote_pit(
            "conversations.messages",
            "conversations messages <conversation-id> [--limit <n>]",
            "List messages for one conversation with redacted message bodies.",
            "ConversationMessagesResult",
            "2",
            &["conversations.messages"],
        ),
        remote_pit(
            "pipelines.list",
            "pipelines list",
            "List sales pipelines in the resolved location.",
            "PipelineListResult",
            "2",
            &["pipelines.list"],
        ),
        remote_pit(
            "pipelines.get",
            "pipelines get <pipeline-id>",
            "Fetch one sales pipeline by filtering the resolved location's pipeline list.",
            "PipelineGetResult",
            "2",
            &["pipelines.list"],
        ),
        remote_pit(
            "opportunities.search",
            "opportunities search [--contact <contact-id>] [--pipeline <pipeline-id>] [--stage <stage-id>] [--status <status>] [--limit <n>]",
            "Search opportunities in the resolved location.",
            "OpportunitySearchResult",
            "2",
            &["opportunities.search"],
        ),
        remote_pit(
            "opportunities.get",
            "opportunities get <opportunity-id>",
            "Fetch one opportunity by id within the resolved location context.",
            "OpportunityGetResult",
            "2",
            &["opportunities.get"],
        ),
        local(
            "completions.bash",
            "completions bash",
            "Generate Bash shell completion script.",
            "ShellCompletion",
            "0",
        ),
        local(
            "completions.zsh",
            "completions zsh",
            "Generate Zsh shell completion script.",
            "ShellCompletion",
            "0",
        ),
        local(
            "completions.fish",
            "completions fish",
            "Generate Fish shell completion script.",
            "ShellCompletion",
            "0",
        ),
        local(
            "completions.powershell",
            "completions powershell",
            "Generate PowerShell completion script.",
            "ShellCompletion",
            "0",
        ),
        local(
            "man",
            "man",
            "Print the CLI manual/help page.",
            "ManualPage",
            "0",
        ),
    ]
}

pub fn command_by_key(command_key: &str) -> Option<CommandMetadata> {
    implemented_commands()
        .into_iter()
        .find(|command| command.command_key == command_key)
}

fn local(
    command_key: &str,
    path: &str,
    summary: &str,
    output_schema: &str,
    phase: &str,
) -> CommandMetadata {
    CommandMetadata {
        command_key: command_key.to_owned(),
        path: path.to_owned(),
        summary: summary.to_owned(),
        phase: phase.to_owned(),
        stability: "stable".to_owned(),
        implemented: true,
        auth_classes: Vec::new(),
        policy_flags: policy_flags_for(command_key),
        endpoint_keys: Vec::new(),
        output_schema: output_schema.to_owned(),
        offline: true,
    }
}

fn remote_pit(
    command_key: &str,
    path: &str,
    summary: &str,
    output_schema: &str,
    phase: &str,
    endpoint_keys: &[&str],
) -> CommandMetadata {
    CommandMetadata {
        command_key: command_key.to_owned(),
        path: path.to_owned(),
        summary: summary.to_owned(),
        phase: phase.to_owned(),
        stability: "stable".to_owned(),
        implemented: true,
        auth_classes: vec!["pit".to_owned()],
        policy_flags: policy_flags_for(command_key),
        endpoint_keys: endpoint_keys.iter().map(|key| (*key).to_owned()).collect(),
        output_schema: output_schema.to_owned(),
        offline: false,
    }
}

fn policy_flags_for(command_key: &str) -> Vec<String> {
    match command_key {
        "profiles.policy.reset" => vec!["confirmation_required".to_owned()],
        _ => Vec::new(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn schema_contains_expected_commands() {
        let schema = command_schema();
        let keys = schema
            .commands
            .iter()
            .map(|command| command.command_key.as_str())
            .collect::<Vec<_>>();

        assert!(keys.contains(&"commands.schema"));
        assert!(keys.contains(&"config.path"));
        assert!(keys.contains(&"auth.pit.add"));
        assert!(keys.contains(&"profiles.list"));
        assert!(keys.contains(&"auth.pit.validate"));
        assert!(keys.contains(&"raw.request"));
        assert!(keys.contains(&"locations.get"));
        assert!(keys.contains(&"locations.list"));
        assert!(keys.contains(&"locations.search"));
        assert!(keys.contains(&"contacts.search"));
        assert!(keys.contains(&"contacts.get"));
        assert!(keys.contains(&"conversations.search"));
        assert!(keys.contains(&"conversations.get"));
        assert!(keys.contains(&"conversations.messages"));
        assert!(keys.contains(&"pipelines.list"));
        assert!(keys.contains(&"pipelines.get"));
        assert!(keys.contains(&"opportunities.search"));
        assert!(keys.contains(&"opportunities.get"));
        assert!(keys.contains(&"errors.list"));
        assert!(keys.contains(&"endpoints.coverage"));
        assert!(keys.contains(&"completions.bash"));
        assert!(keys.contains(&"man"));
    }

    #[test]
    fn local_and_remote_commands_declare_offline_behavior() {
        for command in command_schema().commands {
            assert!(command.implemented, "{}", command.command_key);
            match command.command_key.as_str() {
                "auth.pit.validate"
                | "raw.request"
                | "locations.get"
                | "locations.list"
                | "locations.search"
                | "contacts.search"
                | "contacts.get"
                | "conversations.search"
                | "conversations.get"
                | "conversations.messages"
                | "pipelines.list"
                | "pipelines.get"
                | "opportunities.search"
                | "opportunities.get" => {
                    assert!(!command.offline, "{}", command.command_key);
                    assert_eq!(command.auth_classes, vec!["pit".to_owned()]);
                }
                _ => {
                    assert!(command.offline, "{}", command.command_key);
                    assert!(command.auth_classes.is_empty(), "{}", command.command_key);
                    assert!(command.endpoint_keys.is_empty(), "{}", command.command_key);
                }
            }
        }
    }

    #[test]
    fn command_keys_are_unique() {
        let schema = command_schema();
        let mut keys = schema
            .commands
            .iter()
            .map(|command| command.command_key.clone())
            .collect::<Vec<_>>();
        keys.sort_unstable();
        keys.dedup();

        assert_eq!(keys.len(), schema.commands.len());
    }
}
