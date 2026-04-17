use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::auth::{AuthStatus, auth_status};
use crate::config::{ConfigDoctor, config_doctor};
use crate::endpoints::{EndpointCoverage, EndpointDefinition, endpoint_coverage, find_endpoint};
use crate::errors::{GhlError, Result};
use crate::metadata::{CommandMetadata, command_schema, implemented_commands};
use crate::profiles::redacted_config_with_profiles;
use crate::redaction::redact_json;
use crate::smoke::{SmokeRunOptions, SmokeRunReport, smoke_run};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DoctorReport {
    pub schema_version: u32,
    pub profile: String,
    pub network: bool,
    pub cli: CliDoctorInfo,
    pub config: ConfigDoctor,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub auth: Option<AuthStatus>,
    pub endpoint_coverage: EndpointCoverage,
    pub command_count: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub api: Option<SmokeRunReport>,
    pub warnings: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CliDoctorInfo {
    pub version: String,
    pub os: String,
    pub arch: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct EndpointDoctorReport {
    pub schema_version: u32,
    pub endpoint: EndpointDefinition,
    pub commands: Vec<CommandMetadata>,
    pub safe_probe_available: bool,
    pub notes: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DoctorBundleResult {
    pub path: PathBuf,
    pub redacted: bool,
    pub bytes: u64,
    pub included: Vec<String>,
    pub refused: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
struct SupportBundle {
    schema_version: u32,
    redacted: bool,
    generated_at_unix: u64,
    cli: CliDoctorInfo,
    config: Value,
    config_doctor: ConfigDoctor,
    auth_status: Option<AuthStatus>,
    command_schema: Value,
    endpoint_manifest: Value,
    endpoint_coverage: EndpointCoverage,
    capabilities: Value,
    notes: Vec<String>,
}

pub fn doctor_summary(
    paths: &crate::ConfigPaths,
    profile_name: Option<&str>,
) -> Result<DoctorReport> {
    build_doctor_report(paths, profile_name, None, false)
}

pub fn doctor_api(
    paths: &crate::ConfigPaths,
    profile_name: Option<&str>,
    location_override: Option<&str>,
    company_override: Option<&str>,
    limit: u32,
) -> Result<DoctorReport> {
    let smoke = smoke_run(
        paths,
        profile_name,
        location_override,
        company_override,
        SmokeRunOptions {
            limit,
            skip_optional: true,
            ..SmokeRunOptions::default()
        },
    );
    build_doctor_report(paths, profile_name, Some(smoke), true)
}

pub fn doctor_endpoint(endpoint_key: &str) -> Result<EndpointDoctorReport> {
    let manifest = crate::bundled_manifest()?;
    let endpoint = find_endpoint(&manifest, endpoint_key)
        .ok_or_else(|| GhlError::EndpointNotFound {
            key: endpoint_key.to_owned(),
        })?
        .clone();
    let commands = implemented_commands()
        .into_iter()
        .filter(|command| command.endpoint_keys.iter().any(|key| key == endpoint_key))
        .collect::<Vec<_>>();
    let mut notes = Vec::new();
    if endpoint
        .source_refs
        .iter()
        .any(|source| source.contains("references/"))
    {
        notes.push("endpoint behavior is sourced from local references and must be kept under smoke validation".to_owned());
    }
    if endpoint.risk != "low" {
        notes.push(format!("endpoint risk is `{}`", endpoint.risk));
    }

    Ok(EndpointDoctorReport {
        schema_version: 1,
        safe_probe_available: endpoint.status == "implemented" && !commands.is_empty(),
        endpoint,
        commands,
        notes,
    })
}

pub fn write_support_bundle(
    paths: &crate::ConfigPaths,
    profile_name: Option<&str>,
    location_override: Option<&str>,
    company_override: Option<&str>,
    out: &Path,
    redacted: bool,
) -> Result<DoctorBundleResult> {
    if !redacted {
        return Err(GhlError::Validation {
            message:
                "doctor bundle requires --redacted; unredacted support bundles are not allowed"
                    .to_owned(),
        });
    }

    let bundle = support_bundle(paths, profile_name, location_override, company_override)?;
    let value = serde_json::to_value(bundle)?;
    let redacted_value = redact_json(&value);
    let bytes = serde_json::to_vec_pretty(&redacted_value)?;
    if let Some(parent) = out.parent().filter(|parent| !parent.as_os_str().is_empty()) {
        std::fs::create_dir_all(parent).map_err(|source| GhlError::FileWrite {
            path: parent.to_path_buf(),
            source,
        })?;
    }
    std::fs::write(out, &bytes).map_err(|source| GhlError::FileWrite {
        path: out.to_path_buf(),
        source,
    })?;

    Ok(DoctorBundleResult {
        path: out.to_path_buf(),
        redacted: true,
        bytes: bytes.len() as u64,
        included: vec![
            "cli".to_owned(),
            "redacted_config".to_owned(),
            "config_doctor".to_owned(),
            "auth_status".to_owned(),
            "command_schema".to_owned(),
            "endpoint_manifest".to_owned(),
            "endpoint_coverage".to_owned(),
            "capabilities".to_owned(),
        ],
        refused: vec![
            "credential_store".to_owned(),
            "signet_secret_store".to_owned(),
            "raw_customer_bodies".to_owned(),
            "unredacted_fixtures".to_owned(),
        ],
    })
}

fn build_doctor_report(
    paths: &crate::ConfigPaths,
    profile_name: Option<&str>,
    api: Option<SmokeRunReport>,
    network: bool,
) -> Result<DoctorReport> {
    let manifest = crate::bundled_manifest()?;
    let auth = auth_status(paths, profile_name).ok();
    let profile = auth
        .as_ref()
        .map(|auth| auth.profile.clone())
        .or_else(|| profile_name.map(str::to_owned))
        .unwrap_or_else(|| "default".to_owned());
    let mut warnings = Vec::new();
    if auth.is_none() {
        warnings.push("auth status is unavailable for the selected profile".to_owned());
    }

    Ok(DoctorReport {
        schema_version: 1,
        profile,
        network,
        cli: cli_doctor_info(),
        config: config_doctor(paths),
        auth,
        endpoint_coverage: endpoint_coverage(&manifest),
        command_count: implemented_commands().len(),
        api,
        warnings,
    })
}

fn support_bundle(
    paths: &crate::ConfigPaths,
    profile_name: Option<&str>,
    location_override: Option<&str>,
    company_override: Option<&str>,
) -> Result<SupportBundle> {
    let manifest = crate::bundled_manifest()?;
    let config = redacted_config_with_profiles(paths.clone())?;
    let command_schema = command_schema();
    let capabilities =
        crate::capability_report(paths, profile_name, location_override, company_override)?;

    Ok(SupportBundle {
        schema_version: 1,
        redacted: true,
        generated_at_unix: now_unix(),
        cli: cli_doctor_info(),
        config: serde_json::to_value(config)?,
        config_doctor: config_doctor(paths),
        auth_status: auth_status(paths, profile_name).ok(),
        command_schema: serde_json::to_value(command_schema)?,
        endpoint_manifest: serde_json::to_value(&manifest)?,
        endpoint_coverage: endpoint_coverage(&manifest),
        capabilities: serde_json::to_value(capabilities)?,
        notes: vec![
            "Bundle is JSON, redacted, and intentionally excludes credential stores and raw customer bodies.".to_owned(),
            "Run `ghl doctor api` separately when a live safe-read probe is needed.".to_owned(),
        ],
    })
}

fn cli_doctor_info() -> CliDoctorInfo {
    CliDoctorInfo {
        version: env!("CARGO_PKG_VERSION").to_owned(),
        os: std::env::consts::OS.to_owned(),
        arch: std::env::consts::ARCH.to_owned(),
    }
}

fn now_unix() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs())
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::auth::add_pit;
    use crate::config::resolve_paths;

    #[test]
    fn endpoint_doctor_reports_manifest_and_commands() {
        let report = doctor_endpoint("contacts.search").expect("endpoint");

        assert_eq!(report.endpoint.endpoint_key, "contacts.search");
        assert!(
            report
                .commands
                .iter()
                .any(|command| command.command_key == "contacts.search")
        );
        assert!(report.safe_probe_available);
    }

    #[test]
    fn bundle_requires_redaction() {
        let temp = tempfile::tempdir().expect("tempdir");
        let paths = resolve_paths(Some(temp.path()));
        let out = temp.path().join("bundle.json");

        let error = write_support_bundle(&paths, None, None, None, &out, false)
            .expect_err("redaction required");

        assert_eq!(error.code(), "validation_error");
    }

    #[test]
    fn bundle_does_not_write_plain_pit_secret() {
        let temp = tempfile::tempdir().expect("tempdir");
        let paths = resolve_paths(Some(temp.path()));
        add_pit(
            &paths,
            "default",
            "pit-secret-super-sensitive".to_owned(),
            Some("loc_123".to_owned()),
            None,
            true,
        )
        .expect("pit");
        let out = temp.path().join("bundle.json");

        let result = write_support_bundle(&paths, None, None, None, &out, true).expect("bundle");
        let rendered = std::fs::read_to_string(&out).expect("read bundle");

        assert!(result.bytes > 0);
        assert!(!rendered.contains("pit-secret-super-sensitive"));
        assert!(result.refused.contains(&"credential_store".to_owned()));
    }
}
