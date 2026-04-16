use assert_cmd::Command;
use predicates::prelude::*;
use serde_json::Value;

fn ghl_cli() -> Command {
    Command::cargo_bin("ghl-cli").expect("ghl-cli binary")
}

#[test]
fn help_prints_successfully() {
    ghl_cli()
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("Unofficial Go High Level CLI"));
}

#[test]
fn command_schema_returns_json_envelope() {
    let output = ghl_cli()
        .args(["commands", "schema"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let value: Value = serde_json::from_slice(&output).expect("json");

    assert_eq!(value["ok"], true);
    assert_eq!(value["data"]["schema_version"], 1);
    assert!(
        value["data"]["commands"]
            .as_array()
            .unwrap()
            .iter()
            .any(|command| { command["command_key"] == "commands.schema" })
    );
}

#[test]
fn config_path_honors_config_dir_flag() {
    let temp = tempfile::tempdir().expect("tempdir");
    let output = ghl_cli()
        .arg("--config-dir")
        .arg(temp.path())
        .args(["config", "path"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let value: Value = serde_json::from_slice(&output).expect("json");

    assert_eq!(value["ok"], true);
    assert_eq!(value["data"]["source"], "flag");
    assert_eq!(
        value["data"]["config_dir"].as_str().unwrap(),
        temp.path().to_str().unwrap()
    );
}

#[test]
fn errors_list_contains_validation_error() {
    let output = ghl_cli()
        .args(["errors", "list"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let value: Value = serde_json::from_slice(&output).expect("json");

    assert!(
        value["data"]
            .as_array()
            .unwrap()
            .iter()
            .any(|error| { error["code"] == "validation_error" && error["exit_code"] == 2 })
    );
}

#[test]
fn errors_show_unknown_returns_json_error() {
    let output = ghl_cli()
        .args(["errors", "show", "not_real"])
        .assert()
        .failure()
        .code(2)
        .get_output()
        .stderr
        .clone();
    let value: Value = serde_json::from_slice(&output).expect("json");

    assert_eq!(value["ok"], false);
    assert_eq!(value["error"]["code"], "validation_error");
    assert_eq!(value["error"]["exit_code"], 2);
}

#[test]
fn endpoint_coverage_reports_implemented_read_slice() {
    let output = ghl_cli()
        .args(["endpoints", "coverage"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let value: Value = serde_json::from_slice(&output).expect("json");

    assert_eq!(value["data"]["status"], "scaffold");
    assert_eq!(value["data"]["endpoint_count"], 7);
    assert_eq!(value["data"]["command_mapped_count"], 7);
    assert_eq!(value["data"]["implemented_count"], 7);
}

#[test]
fn bash_completion_is_non_empty() {
    ghl_cli()
        .args(["completions", "bash"])
        .assert()
        .success()
        .stdout(predicate::str::contains("_ghl"));
}

#[test]
fn invalid_command_returns_json_validation_error() {
    let output = ghl_cli()
        .arg("wat")
        .assert()
        .failure()
        .code(2)
        .get_output()
        .stderr
        .clone();
    let value: Value = serde_json::from_slice(&output).expect("json");

    assert_eq!(value["ok"], false);
    assert_eq!(value["error"]["code"], "validation_error");
    assert_eq!(value["error"]["exit_code"], 2);
}

#[test]
fn ghl_alias_binary_works() {
    let output = Command::cargo_bin("ghl")
        .expect("ghl alias binary")
        .args(["commands", "schema", "--pretty"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let value: Value = serde_json::from_slice(&output).expect("json");

    assert_eq!(value["ok"], true);
}

#[test]
fn man_command_is_non_empty() {
    ghl_cli()
        .arg("man")
        .assert()
        .success()
        .stdout(predicate::str::contains("profile persistence"));
}

#[test]
fn pit_add_persists_profile_without_printing_token() {
    let temp = tempfile::tempdir().expect("tempdir");
    let output = ghl_cli()
        .arg("--config-dir")
        .arg(temp.path())
        .arg("--profile")
        .arg("default")
        .args([
            "auth",
            "pit",
            "add",
            "--token-stdin",
            "--location",
            "loc_123",
        ])
        .write_stdin("pit-secret-1234\n")
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let rendered = String::from_utf8_lossy(&output);
    let value: Value = serde_json::from_slice(&output).expect("json");

    assert_eq!(value["ok"], true);
    assert_eq!(value["data"]["profile"], "default");
    assert_eq!(value["data"]["credential_ref"], "pit:default");
    assert!(!rendered.contains("pit-secret-1234"));
}

#[test]
fn pit_list_local_redacts_secret() {
    let temp = tempfile::tempdir().expect("tempdir");
    ghl_cli()
        .arg("--config-dir")
        .arg(temp.path())
        .args([
            "auth",
            "pit",
            "add",
            "--token-stdin",
            "--location",
            "loc_123",
        ])
        .write_stdin("pit-secret-5678\n")
        .assert()
        .success();

    let output = ghl_cli()
        .arg("--config-dir")
        .arg(temp.path())
        .args(["auth", "pit", "list-local"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let rendered = String::from_utf8_lossy(&output);
    let value: Value = serde_json::from_slice(&output).expect("json");

    assert_eq!(
        value["data"]["credentials"][0]["secret_preview"],
        "********5678"
    );
    assert!(!rendered.contains("pit-secret-5678"));
}

#[test]
fn auth_status_reports_local_pit_available() {
    let temp = tempfile::tempdir().expect("tempdir");
    ghl_cli()
        .arg("--config-dir")
        .arg(temp.path())
        .args([
            "auth",
            "pit",
            "add",
            "--token-stdin",
            "--location",
            "loc_123",
        ])
        .write_stdin("pit-secret\n")
        .assert()
        .success();

    let output = ghl_cli()
        .arg("--config-dir")
        .arg(temp.path())
        .args(["auth", "status"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let value: Value = serde_json::from_slice(&output).expect("json");

    assert_eq!(value["data"]["profile"], "default");
    assert_eq!(value["data"]["auth"]["pit"]["available"], true);
    assert_eq!(value["data"]["auth"]["session"]["available"], false);
}

#[test]
fn profiles_list_and_config_show_include_created_profile() {
    let temp = tempfile::tempdir().expect("tempdir");
    ghl_cli()
        .arg("--config-dir")
        .arg(temp.path())
        .arg("--profile")
        .arg("client-a")
        .args(["auth", "pit", "add", "--token-stdin", "--location", "loc_a"])
        .write_stdin("pit-secret\n")
        .assert()
        .success();

    let profiles = ghl_cli()
        .arg("--config-dir")
        .arg(temp.path())
        .args(["profiles", "list"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let profiles: Value = serde_json::from_slice(&profiles).expect("json");
    assert_eq!(profiles["data"]["profiles"][0]["name"], "client-a");
    assert_eq!(profiles["data"]["profiles"][0]["has_pit"], true);

    let config = ghl_cli()
        .arg("--config-dir")
        .arg(temp.path())
        .args(["config", "show"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let config: Value = serde_json::from_slice(&config).expect("json");
    assert_eq!(config["data"]["profiles"][0], "client-a");
}

#[test]
fn policy_reset_requires_confirmation() {
    let temp = tempfile::tempdir().expect("tempdir");
    ghl_cli()
        .arg("--config-dir")
        .arg(temp.path())
        .args([
            "auth",
            "pit",
            "add",
            "--token-stdin",
            "--location",
            "loc_123",
        ])
        .write_stdin("pit-secret\n")
        .assert()
        .success();

    let output = ghl_cli()
        .arg("--config-dir")
        .arg(temp.path())
        .args(["profiles", "policy", "reset", "default"])
        .assert()
        .failure()
        .code(15)
        .get_output()
        .stderr
        .clone();
    let value: Value = serde_json::from_slice(&output).expect("json");

    assert_eq!(value["error"]["code"], "confirmation_required");
}

#[test]
fn raw_request_dry_run_does_not_require_credentials_or_network() {
    let temp = tempfile::tempdir().expect("tempdir");
    let output = ghl_cli()
        .arg("--config-dir")
        .arg(temp.path())
        .args([
            "--dry-run=local",
            "raw",
            "request",
            "--surface",
            "services",
            "--method",
            "get",
            "--path",
            "/locations/loc_123",
        ])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let value: Value = serde_json::from_slice(&output).expect("json");

    assert_eq!(value["data"]["method"], "GET");
    assert_eq!(value["data"]["surface"], "services");
    assert_eq!(value["data"]["network"], false);
}

#[test]
fn locations_get_dry_run_does_not_require_credentials_or_network() {
    let temp = tempfile::tempdir().expect("tempdir");
    let output = ghl_cli()
        .arg("--config-dir")
        .arg(temp.path())
        .args(["--dry-run=local", "locations", "get", "loc_123"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let value: Value = serde_json::from_slice(&output).expect("json");

    assert_eq!(value["data"]["method"], "GET");
    assert_eq!(value["data"]["surface"], "services");
    assert_eq!(value["data"]["path"], "/locations/loc_123");
    assert_eq!(value["data"]["network"], false);
}

#[test]
fn locations_list_dry_run_uses_company_override_without_profile() {
    let temp = tempfile::tempdir().expect("tempdir");
    let output = ghl_cli()
        .arg("--config-dir")
        .arg(temp.path())
        .args([
            "--dry-run=local",
            "locations",
            "list",
            "--company",
            "company_123",
        ])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let value: Value = serde_json::from_slice(&output).expect("json");

    assert_eq!(value["data"]["method"], "GET");
    assert_eq!(
        value["data"]["path"],
        "/locations/search?companyId=company_123&skip=0&limit=50&order=asc"
    );
    assert_eq!(value["data"]["context"]["company_id"]["source"], "override");
    assert_eq!(value["data"]["network"], false);
}

#[test]
fn locations_search_dry_run_maps_query_to_email_filter() {
    let temp = tempfile::tempdir().expect("tempdir");
    let output = ghl_cli()
        .arg("--config-dir")
        .arg(temp.path())
        .args([
            "--dry-run=local",
            "locations",
            "search",
            "test@example.com",
            "--company",
            "company_123",
        ])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let value: Value = serde_json::from_slice(&output).expect("json");

    assert_eq!(value["data"]["email_filter"], "test@example.com");
    assert_eq!(
        value["data"]["path"],
        "/locations/search?companyId=company_123&skip=0&limit=50&order=asc&email=test%40example.com"
    );
}

#[test]
fn contacts_search_dry_run_uses_location_override_and_exact_filter() {
    let temp = tempfile::tempdir().expect("tempdir");
    let output = ghl_cli()
        .arg("--config-dir")
        .arg(temp.path())
        .args([
            "--dry-run=local",
            "--location",
            "loc_123",
            "contacts",
            "search",
            "John",
            "--email",
            "john@example.com",
            "--limit",
            "10",
        ])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let value: Value = serde_json::from_slice(&output).expect("json");

    assert_eq!(value["data"]["method"], "POST");
    assert_eq!(value["data"]["path"], "/contacts/search");
    assert_eq!(
        value["data"]["context"]["location_id"]["source"],
        "override"
    );
    assert_eq!(value["data"]["request_body_json"]["locationId"], "loc_123");
    assert_eq!(value["data"]["request_body_json"]["query"], "John");
    assert_eq!(
        value["data"]["request_body_json"]["filters"]["email"],
        "john@example.com"
    );
    assert_eq!(value["data"]["network"], false);
}

#[test]
fn contacts_get_dry_run_requires_only_location_context() {
    let temp = tempfile::tempdir().expect("tempdir");
    let output = ghl_cli()
        .arg("--config-dir")
        .arg(temp.path())
        .args([
            "--dry-run=local",
            "--location",
            "loc_123",
            "contacts",
            "get",
            "contact_123",
        ])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let value: Value = serde_json::from_slice(&output).expect("json");

    assert_eq!(value["data"]["method"], "GET");
    assert_eq!(value["data"]["path"], "/contacts/contact_123");
    assert_eq!(value["data"]["location_id"], "loc_123");
    assert_eq!(value["data"]["network"], false);
}

#[test]
fn contacts_search_dry_run_rejects_empty_search() {
    let temp = tempfile::tempdir().expect("tempdir");
    let output = ghl_cli()
        .arg("--config-dir")
        .arg(temp.path())
        .args([
            "--dry-run=local",
            "--location",
            "loc_123",
            "contacts",
            "search",
        ])
        .assert()
        .failure()
        .code(2)
        .get_output()
        .stderr
        .clone();
    let value: Value = serde_json::from_slice(&output).expect("json");

    assert_eq!(value["error"]["code"], "validation_error");
}

#[test]
fn conversations_search_dry_run_uses_location_override() {
    let temp = tempfile::tempdir().expect("tempdir");
    let output = ghl_cli()
        .arg("--config-dir")
        .arg(temp.path())
        .args([
            "--dry-run=local",
            "--location",
            "loc_123",
            "conversations",
            "search",
            "--contact",
            "contact_123",
            "--query",
            "Sarah",
            "--status",
            "unread",
            "--limit",
            "10",
        ])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let value: Value = serde_json::from_slice(&output).expect("json");

    assert_eq!(value["data"]["method"], "GET");
    assert_eq!(value["data"]["search_status"], "unread");
    assert_eq!(value["data"]["location_id"], "loc_123");
    assert_eq!(
        value["data"]["path"],
        "/conversations/search?locationId=loc_123&status=unread&limit=10&contactId=contact_123&query=Sarah"
    );
    assert_eq!(value["data"]["network"], false);
}

#[test]
fn conversations_get_dry_run_requires_location_context() {
    let temp = tempfile::tempdir().expect("tempdir");
    let output = ghl_cli()
        .arg("--config-dir")
        .arg(temp.path())
        .args([
            "--dry-run=local",
            "--location",
            "loc_123",
            "conversations",
            "get",
            "conv_123",
        ])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let value: Value = serde_json::from_slice(&output).expect("json");

    assert_eq!(value["data"]["method"], "GET");
    assert_eq!(value["data"]["path"], "/conversations/conv_123");
    assert_eq!(value["data"]["location_id"], "loc_123");
    assert_eq!(value["data"]["network"], false);
}

#[test]
fn conversations_messages_dry_run_supports_pagination_filters() {
    let temp = tempfile::tempdir().expect("tempdir");
    let output = ghl_cli()
        .arg("--config-dir")
        .arg(temp.path())
        .args([
            "--dry-run=local",
            "--location",
            "loc_123",
            "conversations",
            "messages",
            "conv_123",
            "--limit",
            "10",
            "--last-message-id",
            "msg_099",
            "--message-type",
            "TYPE_SMS",
        ])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let value: Value = serde_json::from_slice(&output).expect("json");

    assert_eq!(value["data"]["method"], "GET");
    assert_eq!(
        value["data"]["path"],
        "/conversations/conv_123/messages?limit=10&lastMessageId=msg_099&type=TYPE_SMS"
    );
    assert_eq!(value["data"]["location_id"], "loc_123");
    assert_eq!(value["data"]["network"], false);
}

#[test]
fn profiles_set_default_company_persists_context() {
    let temp = tempfile::tempdir().expect("tempdir");
    ghl_cli()
        .arg("--config-dir")
        .arg(temp.path())
        .args([
            "auth",
            "pit",
            "add",
            "--token-stdin",
            "--location",
            "loc_123",
        ])
        .write_stdin("pit-secret\n")
        .assert()
        .success();

    let output = ghl_cli()
        .arg("--config-dir")
        .arg(temp.path())
        .args(["profiles", "set-default-company", "default", "company_123"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let value: Value = serde_json::from_slice(&output).expect("json");
    assert_eq!(value["data"]["company_id"], "company_123");

    let output = ghl_cli()
        .arg("--config-dir")
        .arg(temp.path())
        .args(["profiles", "show", "default"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let value: Value = serde_json::from_slice(&output).expect("json");
    assert_eq!(value["data"]["company_id"], "company_123");
}

#[test]
fn offline_blocks_network_commands_without_dry_run() {
    let temp = tempfile::tempdir().expect("tempdir");
    let output = ghl_cli()
        .arg("--config-dir")
        .arg(temp.path())
        .args([
            "--offline",
            "raw",
            "request",
            "--surface",
            "services",
            "--method",
            "get",
            "--path",
            "/locations/loc_123",
        ])
        .assert()
        .failure()
        .code(17)
        .get_output()
        .stderr
        .clone();
    let value: Value = serde_json::from_slice(&output).expect("json");

    assert_eq!(value["error"]["code"], "offline_blocked");

    ghl_cli()
        .arg("--config-dir")
        .arg(temp.path())
        .args([
            "--offline",
            "--dry-run=local",
            "raw",
            "request",
            "--surface",
            "services",
            "--method",
            "get",
            "--path",
            "/locations/loc_123",
        ])
        .assert()
        .success();
}

#[test]
fn command_schema_includes_raw_and_pit_validate() {
    let output = ghl_cli()
        .args(["commands", "schema"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let value: Value = serde_json::from_slice(&output).expect("json");
    let commands = value["data"]["commands"].as_array().unwrap();

    assert!(
        commands
            .iter()
            .any(|command| command["command_key"] == "raw.request")
    );
    assert!(
        commands
            .iter()
            .any(|command| command["command_key"] == "auth.pit.validate")
    );
    assert!(
        commands
            .iter()
            .any(|command| command["command_key"] == "locations.get")
    );
    assert!(
        commands
            .iter()
            .any(|command| command["command_key"] == "locations.list")
    );
    assert!(
        commands
            .iter()
            .any(|command| command["command_key"] == "locations.search")
    );
    assert!(
        commands
            .iter()
            .any(|command| command["command_key"] == "contacts.search")
    );
    assert!(
        commands
            .iter()
            .any(|command| command["command_key"] == "contacts.get")
    );
    assert!(
        commands
            .iter()
            .any(|command| command["command_key"] == "conversations.search")
    );
    assert!(
        commands
            .iter()
            .any(|command| command["command_key"] == "conversations.get")
    );
    assert!(
        commands
            .iter()
            .any(|command| command["command_key"] == "conversations.messages")
    );
}
