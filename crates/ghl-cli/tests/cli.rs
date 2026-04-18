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
    assert_eq!(value["data"]["endpoint_count"], 25);
    assert_eq!(value["data"]["command_mapped_count"], 25);
    assert_eq!(value["data"]["implemented_count"], 25);
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
        value["data"]["request_body_json"]["filters"][0]["field"],
        "email"
    );
    assert_eq!(
        value["data"]["request_body_json"]["filters"][0]["operator"],
        "eq"
    );
    assert_eq!(
        value["data"]["request_body_json"]["filters"][0]["value"],
        "john@example.com"
    );
    assert_eq!(value["data"]["network"], false);
}

#[test]
fn contacts_list_dry_run_returns_summary_request_without_search_term() {
    let temp = tempfile::tempdir().expect("tempdir");
    let output = ghl_cli()
        .arg("--config-dir")
        .arg(temp.path())
        .args([
            "--dry-run=local",
            "--location",
            "loc_123",
            "contacts",
            "list",
            "--limit",
            "5",
        ])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let value: Value = serde_json::from_slice(&output).expect("json");

    assert_eq!(value["data"]["method"], "POST");
    assert_eq!(value["data"]["path"], "/contacts/search");
    assert_eq!(value["data"]["request_body_json"]["locationId"], "loc_123");
    assert_eq!(value["data"]["request_body_json"]["pageLimit"], 5);
    assert!(value["data"]["request_body_json"].get("query").is_none());
    assert!(value["data"]["request_body_json"].get("filters").is_none());
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
fn pipelines_list_dry_run_uses_location_context() {
    let temp = tempfile::tempdir().expect("tempdir");
    let output = ghl_cli()
        .arg("--config-dir")
        .arg(temp.path())
        .args([
            "--dry-run=local",
            "--location",
            "loc_123",
            "pipelines",
            "list",
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
        "/opportunities/pipelines?locationId=loc_123"
    );
    assert_eq!(value["data"]["location_id"], "loc_123");
    assert_eq!(value["data"]["network"], false);
}

#[test]
fn pipelines_get_dry_run_filters_list_client_side() {
    let temp = tempfile::tempdir().expect("tempdir");
    let output = ghl_cli()
        .arg("--config-dir")
        .arg(temp.path())
        .args([
            "--dry-run=local",
            "--location",
            "loc_123",
            "pipelines",
            "get",
            "pipe_123",
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
        "/opportunities/pipelines?locationId=loc_123"
    );
    assert_eq!(value["data"]["pipeline_id"], "pipe_123");
    assert_eq!(value["data"]["network"], false);
}

#[test]
fn opportunities_search_dry_run_uses_location_and_filters() {
    let temp = tempfile::tempdir().expect("tempdir");
    let output = ghl_cli()
        .arg("--config-dir")
        .arg(temp.path())
        .args([
            "--dry-run=local",
            "--location",
            "loc_123",
            "opportunities",
            "search",
            "--query",
            "Roof",
            "--pipeline",
            "pipe_123",
            "--stage",
            "stage_123",
            "--contact",
            "contact_123",
            "--status",
            "open",
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
    assert_eq!(value["data"]["opportunity_status"], "open");
    assert_eq!(
        value["data"]["path"],
        "/opportunities/search?location_id=loc_123&limit=10&q=Roof&pipeline_id=pipe_123&pipeline_stage_id=stage_123&contact_id=contact_123&status=open"
    );
    assert_eq!(value["data"]["network"], false);
}

#[test]
fn opportunities_get_dry_run_requires_location_context() {
    let temp = tempfile::tempdir().expect("tempdir");
    let output = ghl_cli()
        .arg("--config-dir")
        .arg(temp.path())
        .args([
            "--dry-run=local",
            "--location",
            "loc_123",
            "opportunities",
            "get",
            "opp_123",
        ])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let value: Value = serde_json::from_slice(&output).expect("json");

    assert_eq!(value["data"]["method"], "GET");
    assert_eq!(value["data"]["path"], "/opportunities/opp_123");
    assert_eq!(value["data"]["location_id"], "loc_123");
    assert_eq!(value["data"]["network"], false);
}

#[test]
fn calendars_list_dry_run_uses_location_context() {
    let temp = tempfile::tempdir().expect("tempdir");
    let output = ghl_cli()
        .arg("--config-dir")
        .arg(temp.path())
        .args([
            "--dry-run=local",
            "--location",
            "loc_123",
            "calendars",
            "list",
            "--group",
            "group_123",
            "--show-drafted",
            "false",
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
        "/calendars/?locationId=loc_123&groupId=group_123&showDrafted=false"
    );
    assert_eq!(value["data"]["network"], false);
}

#[test]
fn calendars_events_dry_run_builds_date_range_without_event_bodies() {
    let temp = tempfile::tempdir().expect("tempdir");
    let output = ghl_cli()
        .arg("--config-dir")
        .arg(temp.path())
        .args([
            "--dry-run=local",
            "--location",
            "loc_123",
            "calendars",
            "events",
            "--calendar",
            "cal_123",
            "--date",
            "2026-02-27",
        ])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let value: Value = serde_json::from_slice(&output).expect("json");

    assert_eq!(value["data"]["method"], "GET");
    assert_eq!(value["data"]["start_time"], 1772150400000u64);
    assert_eq!(value["data"]["end_time"], 1772236800000u64);
    assert_eq!(
        value["data"]["path"],
        "/calendars/events?locationId=loc_123&startTime=1772150400000&endTime=1772236800000&calendarId=cal_123"
    );
    assert_eq!(value["data"]["network"], false);
}

#[test]
fn calendars_free_slots_dry_run_builds_slot_query() {
    let temp = tempfile::tempdir().expect("tempdir");
    let output = ghl_cli()
        .arg("--config-dir")
        .arg(temp.path())
        .args([
            "--dry-run=local",
            "--location",
            "loc_123",
            "calendars",
            "free-slots",
            "--calendar",
            "cal_123",
            "--date",
            "2026-02-27",
            "--timezone",
            "America/Denver",
            "--enable-look-busy",
        ])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let value: Value = serde_json::from_slice(&output).expect("json");

    assert_eq!(value["data"]["method"], "GET");
    assert_eq!(value["data"]["calendar_id"], "cal_123");
    assert_eq!(
        value["data"]["path"],
        "/calendars/cal_123/free-slots?startDate=1772150400000&endDate=1772236800000&timezone=America%2FDenver&enableLookBusy=true"
    );
    assert_eq!(value["data"]["network"], false);
}

#[test]
fn users_list_dry_run_uses_location_context() {
    let temp = tempfile::tempdir().expect("tempdir");
    let output = ghl_cli()
        .arg("--config-dir")
        .arg(temp.path())
        .args([
            "--dry-run=local",
            "--location",
            "loc_123",
            "users",
            "list",
            "--limit",
            "5",
        ])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let value: Value = serde_json::from_slice(&output).expect("json");

    assert_eq!(value["data"]["method"], "GET");
    assert_eq!(value["data"]["path"], "/users/?locationId=loc_123");
    assert_eq!(value["data"]["network"], false);
}

#[test]
fn teams_list_dry_run_aliases_users_endpoint() {
    let temp = tempfile::tempdir().expect("tempdir");
    let output = ghl_cli()
        .arg("--config-dir")
        .arg(temp.path())
        .args([
            "--dry-run=local",
            "--location",
            "loc_123",
            "teams",
            "list",
            "--limit",
            "5",
        ])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let value: Value = serde_json::from_slice(&output).expect("json");

    assert_eq!(value["data"]["method"], "GET");
    assert_eq!(value["data"]["path"], "/users/?locationId=loc_123");
    assert_eq!(value["data"]["network"], false);
}

#[test]
fn users_get_dry_run_requires_location_context() {
    let temp = tempfile::tempdir().expect("tempdir");
    let output = ghl_cli()
        .arg("--config-dir")
        .arg(temp.path())
        .args([
            "--dry-run=local",
            "--location",
            "loc_123",
            "users",
            "get",
            "user_123",
        ])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let value: Value = serde_json::from_slice(&output).expect("json");

    assert_eq!(value["data"]["method"], "GET");
    assert_eq!(value["data"]["path"], "/users/user_123");
    assert_eq!(value["data"]["location_id"], "loc_123");
    assert_eq!(value["data"]["network"], false);
}

#[test]
fn users_search_dry_run_supports_email_and_query_modes() {
    let temp = tempfile::tempdir().expect("tempdir");
    let by_email = ghl_cli()
        .arg("--config-dir")
        .arg(temp.path())
        .args([
            "--dry-run=local",
            "--location",
            "loc_123",
            "users",
            "search",
            "--email",
            "person@example.com",
        ])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let by_email: Value = serde_json::from_slice(&by_email).expect("json");
    assert_eq!(by_email["data"]["method"], "POST");
    assert_eq!(by_email["data"]["path"], "/users/search/filter-by-email");
    assert_eq!(by_email["data"]["search_mode"], "email");
    assert_eq!(
        by_email["data"]["request_body_json"]["locationId"],
        "loc_123"
    );

    let by_query = ghl_cli()
        .arg("--config-dir")
        .arg(temp.path())
        .args([
            "--dry-run=local",
            "--company",
            "company_123",
            "users",
            "search",
            "--query",
            "Person",
            "--limit",
            "10",
        ])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let by_query: Value = serde_json::from_slice(&by_query).expect("json");
    assert_eq!(by_query["data"]["method"], "GET");
    assert_eq!(by_query["data"]["search_mode"], "query");
    assert_eq!(
        by_query["data"]["path"],
        "/users/search?companyId=company_123&query=Person&skip=0&limit=10"
    );
}

#[test]
fn smoke_run_dry_run_reports_statuses_without_customer_data() {
    let temp = tempfile::tempdir().expect("tempdir");
    let output = ghl_cli()
        .arg("--config-dir")
        .arg(temp.path())
        .args([
            "--dry-run=local",
            "--location",
            "loc_123",
            "--company",
            "company_123",
            "smoke",
            "run",
            "--limit",
            "5",
            "--contact-email",
            "person@example.com",
            "--contact-id",
            "contact_123",
            "--conversation-id",
            "conv_123",
            "--pipeline-id",
            "pipe_123",
            "--opportunity-id",
            "opp_123",
            "--calendar-id",
            "cal_123",
            "--calendar-date",
            "2026-02-27",
            "--calendar-timezone",
            "America/Denver",
            "--user-id",
            "user_123",
            "--user-email",
            "person@example.com",
        ])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let rendered = String::from_utf8_lossy(&output);
    let value: Value = serde_json::from_slice(&output).expect("json");
    let checks = value["data"]["checks"].as_array().unwrap();

    assert_eq!(value["data"]["ok"], true);
    assert_eq!(value["data"]["mode"], "dry_run");
    assert_eq!(value["data"]["location_id"], "loc_123");
    assert_eq!(value["data"]["company_id"], "company_123");
    assert!(checks.iter().any(|check| check["name"] == "locations.get"
        && check["status"] == "planned"
        && check["required"] == true));
    assert!(checks.iter().any(|check| check["name"] == "contacts.search"
        && check["status"] == "planned"
        && check["required"] == false));
    assert!(checks.iter().any(|check| check["name"] == "calendars.list"
        && check["status"] == "planned"
        && check["required"] == true));
    assert!(
        checks
            .iter()
            .any(|check| check["name"] == "calendars.free_slots"
                && check["status"] == "planned"
                && check["required"] == false)
    );
    assert!(checks.iter().any(|check| check["name"] == "users.list"
        && check["status"] == "planned"
        && check["required"] == true));
    assert!(checks.iter().any(|check| check["name"] == "users.search"
        && check["status"] == "planned"
        && check["required"] == false));
    assert!(!rendered.contains("person@example.com"));
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
            .any(|command| command["command_key"] == "doctor.summary")
    );
    assert!(
        commands
            .iter()
            .any(|command| command["command_key"] == "doctor.api")
    );
    assert!(
        commands
            .iter()
            .any(|command| command["command_key"] == "doctor.bundle")
    );
    assert!(
        commands
            .iter()
            .any(|command| command["command_key"] == "capabilities.list")
    );
    assert!(
        commands
            .iter()
            .any(|command| command["command_key"] == "capabilities.check")
    );
    assert!(
        commands
            .iter()
            .any(|command| command["command_key"] == "audit.list")
    );
    assert!(
        commands
            .iter()
            .any(|command| command["command_key"] == "audit.show")
    );
    assert!(
        commands
            .iter()
            .any(|command| command["command_key"] == "audit.export")
    );
    assert!(
        commands
            .iter()
            .any(|command| command["command_key"] == "idempotency.list")
    );
    assert!(
        commands
            .iter()
            .any(|command| command["command_key"] == "idempotency.show")
    );
    assert!(
        commands
            .iter()
            .any(|command| command["command_key"] == "idempotency.clear")
    );
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
            .any(|command| command["command_key"] == "contacts.list")
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
    assert!(
        commands
            .iter()
            .any(|command| command["command_key"] == "pipelines.list")
    );
    assert!(
        commands
            .iter()
            .any(|command| command["command_key"] == "pipelines.get")
    );
    assert!(
        commands
            .iter()
            .any(|command| command["command_key"] == "opportunities.search")
    );
    assert!(
        commands
            .iter()
            .any(|command| command["command_key"] == "opportunities.get")
    );
    assert!(
        commands
            .iter()
            .any(|command| command["command_key"] == "calendars.list")
    );
    assert!(
        commands
            .iter()
            .any(|command| command["command_key"] == "calendars.get")
    );
    assert!(
        commands
            .iter()
            .any(|command| command["command_key"] == "calendars.events")
    );
    assert!(
        commands
            .iter()
            .any(|command| command["command_key"] == "calendars.free_slots")
    );
    assert!(
        commands
            .iter()
            .any(|command| command["command_key"] == "appointments.create")
    );
    assert!(
        commands
            .iter()
            .any(|command| command["command_key"] == "appointments.update")
    );
    assert!(
        commands
            .iter()
            .any(|command| command["command_key"] == "appointments.cancel")
    );
    assert!(
        commands
            .iter()
            .any(|command| command["command_key"] == "appointments.notes.list")
    );
    assert!(
        commands
            .iter()
            .any(|command| command["command_key"] == "appointments.notes.create")
    );
    assert!(
        commands
            .iter()
            .any(|command| command["command_key"] == "appointments.notes.update")
    );
    assert!(
        commands
            .iter()
            .any(|command| command["command_key"] == "appointments.notes.delete")
    );
    assert!(
        commands
            .iter()
            .any(|command| command["command_key"] == "users.list")
    );
    assert!(
        commands
            .iter()
            .any(|command| command["command_key"] == "users.get")
    );
    assert!(
        commands
            .iter()
            .any(|command| command["command_key"] == "users.search")
    );
    assert!(
        commands
            .iter()
            .any(|command| command["command_key"] == "teams.list")
    );
    assert!(
        commands
            .iter()
            .any(|command| command["command_key"] == "smoke.run")
    );
}

#[test]
fn doctor_summary_reports_local_diagnostics() {
    let temp = tempfile::tempdir().expect("tempdir");
    let output = ghl_cli()
        .arg("--config-dir")
        .arg(temp.path())
        .arg("doctor")
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let value: Value = serde_json::from_slice(&output).expect("json");

    assert_eq!(value["data"]["network"], false);
    assert_eq!(value["data"]["schema_version"], 1);
    assert!(
        !value["data"]["config"]["paths"]
            .as_array()
            .unwrap()
            .is_empty()
    );
    assert!(value["data"]["command_count"].as_u64().unwrap() > 0);
}

#[test]
fn doctor_endpoint_reports_mapped_commands() {
    let output = ghl_cli()
        .args(["doctor", "endpoint", "contacts.search"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let value: Value = serde_json::from_slice(&output).expect("json");

    assert_eq!(value["data"]["endpoint"]["endpoint_key"], "contacts.search");
    assert_eq!(value["data"]["safe_probe_available"], true);
    assert!(
        value["data"]["commands"]
            .as_array()
            .unwrap()
            .iter()
            .any(|command| command["command_key"] == "contacts.search")
    );
}

#[test]
fn capabilities_check_reports_policy_block_for_planned_write() {
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
        .write_stdin(
            "pit-secret
",
        )
        .assert()
        .success();

    let output = ghl_cli()
        .arg("--config-dir")
        .arg(temp.path())
        .args(["capabilities", "check", "contacts.write"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let value: Value = serde_json::from_slice(&output).expect("json");

    assert_eq!(value["data"]["capability"], "contacts.write");
    assert_eq!(value["data"]["state"], "blocked_by_policy");
    assert_eq!(value["data"]["confidence"], "known");
}

#[test]
fn capabilities_list_reports_expected_available_read() {
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
        .write_stdin(
            "pit-secret
",
        )
        .assert()
        .success();

    let output = ghl_cli()
        .arg("--config-dir")
        .arg(temp.path())
        .args(["capabilities"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let value: Value = serde_json::from_slice(&output).expect("json");

    assert_eq!(
        value["data"]["capabilities"]["contacts.list"]["state"],
        "expected_available"
    );
}

#[test]
fn doctor_bundle_requires_redacted_flag_and_writes_safe_json() {
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
        .write_stdin(
            "pit-secret-bundle
",
        )
        .assert()
        .success();
    let out = temp.path().join("support-bundle.json");

    let error = ghl_cli()
        .arg("--config-dir")
        .arg(temp.path())
        .args(["doctor", "bundle", "--out"])
        .arg(&out)
        .assert()
        .failure()
        .code(2)
        .get_output()
        .stderr
        .clone();
    let value: Value = serde_json::from_slice(&error).expect("json");
    assert_eq!(value["error"]["code"], "validation_error");

    let output = ghl_cli()
        .arg("--config-dir")
        .arg(temp.path())
        .args(["doctor", "bundle", "--out"])
        .arg(&out)
        .arg("--redacted")
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let value: Value = serde_json::from_slice(&output).expect("json");
    let rendered = std::fs::read_to_string(&out).expect("bundle");

    assert_eq!(value["data"]["redacted"], true);
    assert!(out.exists());
    assert!(!rendered.contains("pit-secret-bundle"));
    assert!(rendered.contains("endpoint_manifest"));
}

#[test]
fn audit_list_show_and_export_read_local_redacted_journal() {
    let temp = tempfile::tempdir().expect("tempdir");
    let audit_dir = temp.path().join("data/audit");
    std::fs::create_dir_all(&audit_dir).expect("audit dir");
    let journal = audit_dir.join("audit.jsonl");
    std::fs::write(
        &journal,
        r#"{"schema_version":1,"id":"audit-test-1","timestamp_unix_ms":1000,"profile":"default","location_id":"loc_123","command":"contacts.create","action_class":"write","dry_run":false,"policy_flags":["confirmation_required"],"resource":{"resource_type":"contact","id":"contact_123"},"request_summary":{"token":"[REDACTED]","email":"person@example.com"},"result":{"status":"success","resource_id":"contact_123"}}
"#,
    )
    .expect("journal write");

    let list = ghl_cli()
        .arg("--config-dir")
        .arg(temp.path())
        .args([
            "audit",
            "list",
            "--action",
            "write",
            "--resource",
            "contact_123",
        ])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let list_value: Value = serde_json::from_slice(&list).expect("json");
    assert_eq!(list_value["data"]["count"], 1);
    assert_eq!(list_value["data"]["entries"][0]["id"], "audit-test-1");

    let show = ghl_cli()
        .arg("--config-dir")
        .arg(temp.path())
        .args(["audit", "show", "audit-test-1"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let rendered = String::from_utf8_lossy(&show);
    let show_value: Value = serde_json::from_slice(&show).expect("json");
    assert_eq!(show_value["data"]["entry"]["resource"]["id"], "contact_123");
    assert!(!rendered.contains("pit-secret"));

    let out = temp.path().join("audit-export.json");
    let export = ghl_cli()
        .arg("--config-dir")
        .arg(temp.path())
        .args(["audit", "export", "--out"])
        .arg(&out)
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let export_value: Value = serde_json::from_slice(&export).expect("json");
    assert_eq!(export_value["data"]["count"], 1);
    assert!(out.exists());
}

#[test]
fn idempotency_list_show_and_clear_manage_local_cache() {
    let temp = tempfile::tempdir().expect("tempdir");
    let store_dir = temp.path().join("data/idempotency");
    std::fs::create_dir_all(&store_dir).expect("idempotency dir");
    let store = store_dir.join("idempotency.jsonl");
    std::fs::write(
        &store,
        r#"{"schema_version":1,"key":"create-contact-1","scoped_key":"default:loc_123:contacts.create:create-contact-1","profile":"default","location_id":"loc_123","command":"contacts.create","request_hash":"fnv1a64:abc","status":"succeeded","resource_id":"contact_123","audit_entry_id":"audit-test-1","created_at_unix_ms":1000,"updated_at_unix_ms":1000}
"#,
    )
    .expect("store write");

    let list = ghl_cli()
        .arg("--config-dir")
        .arg(temp.path())
        .args(["idempotency", "list"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let list_value: Value = serde_json::from_slice(&list).expect("json");
    assert_eq!(list_value["data"]["count"], 1);

    let show = ghl_cli()
        .arg("--config-dir")
        .arg(temp.path())
        .args(["idempotency", "show", "create-contact-1"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let show_value: Value = serde_json::from_slice(&show).expect("json");
    assert_eq!(show_value["data"]["record"]["resource_id"], "contact_123");

    let error = ghl_cli()
        .arg("--config-dir")
        .arg(temp.path())
        .args(["idempotency", "clear", "create-contact-1"])
        .assert()
        .failure()
        .code(15)
        .get_output()
        .stderr
        .clone();
    let error_value: Value = serde_json::from_slice(&error).expect("json");
    assert_eq!(error_value["error"]["code"], "confirmation_required");

    let cleared = ghl_cli()
        .arg("--config-dir")
        .arg(temp.path())
        .args(["idempotency", "clear", "create-contact-1", "--yes"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let cleared_value: Value = serde_json::from_slice(&cleared).expect("json");
    assert_eq!(cleared_value["data"]["removed"], true);
    assert_eq!(cleared_value["data"]["remaining_count"], 0);
}

#[test]
fn appointments_create_dry_run_writes_audit_and_plans_preflight() {
    let temp = tempfile::tempdir().expect("tempdir");
    let output = ghl_cli()
        .arg("--config-dir")
        .arg(temp.path())
        .args([
            "--dry-run=local",
            "--location",
            "loc_123",
            "appointments",
            "create",
            "--calendar",
            "cal_123",
            "--contact",
            "contact_123",
            "--starts-at",
            "2026-02-27T09:00:00-07:00",
            "--ends-at",
            "2026-02-27T09:30:00-07:00",
            "--title",
            "Discovery Call",
            "--assigned-user",
            "user_123",
            "--meeting-location-type",
            "phone",
            "--timezone",
            "America/Denver",
            "--idempotency-key",
            "appt-key-1",
            "--no-notify",
        ])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let value: Value = serde_json::from_slice(&output).expect("json");

    assert_eq!(value["data"]["method"], "POST");
    assert_eq!(value["data"]["path"], "/calendars/events/appointments");
    assert_eq!(value["data"]["request_body_json"]["locationId"], "loc_123");
    assert_eq!(value["data"]["request_body_json"]["toNotify"], false);
    assert_eq!(
        value["data"]["preflight"]["free_slot_check"]["status"],
        "planned"
    );
    assert_eq!(value["data"]["network"], false);

    let audit =
        std::fs::read_to_string(temp.path().join("data/audit/audit.jsonl")).expect("audit journal");
    assert!(audit.contains("appointments.create"));
}

#[test]
fn appointments_create_real_requires_confirmation() {
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
        .args([
            "appointments",
            "create",
            "--calendar",
            "cal_123",
            "--contact",
            "contact_123",
            "--starts-at",
            "2026-02-27T09:00:00-07:00",
            "--ends-at",
            "2026-02-27T09:30:00-07:00",
            "--idempotency-key",
            "appt-key-1",
            "--skip-free-slot-check",
        ])
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
fn appointments_update_dry_run_writes_audit() {
    let temp = tempfile::tempdir().expect("tempdir");
    let output = ghl_cli()
        .arg("--config-dir")
        .arg(temp.path())
        .args([
            "--dry-run=local",
            "--location",
            "loc_123",
            "appointments",
            "update",
            "appt_123",
            "--title",
            "Updated Discovery Call",
            "--status",
            "confirmed",
            "--starts-at",
            "2026-02-27T10:00:00-07:00",
            "--ends-at",
            "2026-02-27T10:30:00-07:00",
            "--no-notify",
            "--idempotency-key",
            "update-appt-1",
        ])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let value: Value = serde_json::from_slice(&output).expect("json");

    assert_eq!(value["data"]["method"], "PUT");
    assert_eq!(
        value["data"]["path"],
        "/calendars/events/appointments/appt_123"
    );
    assert_eq!(
        value["data"]["request_body_json"]["appointmentStatus"],
        "confirmed"
    );
    assert_eq!(value["data"]["request_body_json"]["toNotify"], false);
    assert_eq!(value["data"]["network"], false);

    let audit =
        std::fs::read_to_string(temp.path().join("data/audit/audit.jsonl")).expect("audit journal");
    assert!(audit.contains("appointments.update"));
}

#[test]
fn appointments_cancel_dry_run_writes_audit() {
    let temp = tempfile::tempdir().expect("tempdir");
    let output = ghl_cli()
        .arg("--config-dir")
        .arg(temp.path())
        .args([
            "--dry-run=local",
            "--location",
            "loc_123",
            "appointments",
            "cancel",
            "appt_123",
            "--idempotency-key",
            "cancel-appt-1",
        ])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let value: Value = serde_json::from_slice(&output).expect("json");

    assert_eq!(value["data"]["method"], "DELETE");
    assert_eq!(
        value["data"]["path"],
        "/calendars/events/appointments/appt_123"
    );
    assert_eq!(value["data"]["network"], false);

    let audit =
        std::fs::read_to_string(temp.path().join("data/audit/audit.jsonl")).expect("audit journal");
    assert!(audit.contains("appointments.cancel"));
}

#[test]
fn appointments_update_real_requires_confirmation() {
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
        .args([
            "appointments",
            "update",
            "appt_123",
            "--title",
            "Updated Discovery Call",
            "--idempotency-key",
            "update-appt-1",
        ])
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
fn appointments_cancel_real_requires_confirmation() {
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
        .args([
            "appointments",
            "cancel",
            "appt_123",
            "--idempotency-key",
            "cancel-appt-1",
        ])
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
fn appointments_notes_list_dry_run_builds_request() {
    let temp = tempfile::tempdir().expect("tempdir");
    let output = ghl_cli()
        .arg("--config-dir")
        .arg(temp.path())
        .args([
            "--dry-run=local",
            "--location",
            "loc_123",
            "appointments",
            "notes",
            "list",
            "appt_123",
            "--limit",
            "5",
            "--offset",
            "2",
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
        "/calendars/events/appointments/appt_123/notes?limit=5&offset=2"
    );
    assert_eq!(value["data"]["network"], false);
}

#[test]
fn appointments_notes_create_dry_run_redacts_body_and_writes_audit() {
    let temp = tempfile::tempdir().expect("tempdir");
    let output = ghl_cli()
        .arg("--config-dir")
        .arg(temp.path())
        .args([
            "--dry-run=local",
            "--location",
            "loc_123",
            "appointments",
            "notes",
            "create",
            "appt_123",
            "--body",
            "private note",
            "--user",
            "user_123",
            "--idempotency-key",
            "create-note-1",
        ])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let value: Value = serde_json::from_slice(&output).expect("json");

    assert_eq!(value["data"]["method"], "POST");
    assert_eq!(value["data"]["request_body_json"]["body"], "[REDACTED]");
    assert_eq!(value["data"]["network"], false);

    let audit =
        std::fs::read_to_string(temp.path().join("data/audit/audit.jsonl")).expect("audit journal");
    assert!(audit.contains("appointments.notes.create"));
    assert!(!audit.contains("private note"));
}

#[test]
fn appointments_notes_update_dry_run_accepts_body_file() {
    let temp = tempfile::tempdir().expect("tempdir");
    let note_path = temp.path().join("note.txt");
    std::fs::write(&note_path, "private note from file").expect("write note");

    let output = ghl_cli()
        .arg("--config-dir")
        .arg(temp.path())
        .args([
            "--dry-run=local",
            "--location",
            "loc_123",
            "appointments",
            "notes",
            "update",
            "appt_123",
            "note_123",
            "--from-file",
        ])
        .arg(&note_path)
        .args(["--idempotency-key", "update-note-1"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let value: Value = serde_json::from_slice(&output).expect("json");

    assert_eq!(value["data"]["method"], "PUT");
    assert_eq!(value["data"]["request_body_json"]["body"], "[REDACTED]");
    assert_eq!(value["data"]["note_id"], "note_123");
}

#[test]
fn appointments_notes_delete_real_requires_confirmation() {
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
        .args([
            "appointments",
            "notes",
            "delete",
            "appt_123",
            "note_123",
            "--idempotency-key",
            "delete-note-1",
        ])
        .assert()
        .failure()
        .code(15)
        .get_output()
        .stderr
        .clone();
    let value: Value = serde_json::from_slice(&output).expect("json");

    assert_eq!(value["error"]["code"], "confirmation_required");
}
