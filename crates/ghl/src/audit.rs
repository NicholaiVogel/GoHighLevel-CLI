use std::fs::{self, File, OpenOptions};
use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

use chrono::DateTime;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

use crate::config::ConfigPaths;
use crate::errors::{GhlError, Result};
use crate::redaction::redact_json;

static AUDIT_COUNTER: AtomicU64 = AtomicU64::new(0);

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AuditEntry {
    pub schema_version: u32,
    pub id: String,
    pub timestamp_unix_ms: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub profile: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub company_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub location_id: Option<String>,
    pub command: String,
    pub action_class: String,
    pub dry_run: bool,
    pub policy_flags: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub resource: Option<AuditResource>,
    pub request_summary: Value,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub upstream: Option<AuditUpstreamSummary>,
    pub result: AuditResultSummary,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AuditResource {
    pub resource_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AuditUpstreamSummary {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub request_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status_code: Option<u16>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub endpoint_key: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AuditResultSummary {
    pub status: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub resource_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AuditEntryInput {
    pub profile: Option<String>,
    pub company_id: Option<String>,
    pub location_id: Option<String>,
    pub command: String,
    pub action_class: String,
    pub dry_run: bool,
    pub policy_flags: Vec<String>,
    pub resource: Option<AuditResource>,
    pub request_summary: Value,
    pub upstream: Option<AuditUpstreamSummary>,
    pub result: AuditResultSummary,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct AuditListOptions {
    pub from_unix_ms: Option<u64>,
    pub to_unix_ms: Option<u64>,
    pub action: Option<String>,
    pub resource: Option<String>,
    pub limit: Option<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AuditListResult {
    pub schema_version: u32,
    pub journal_path: PathBuf,
    pub count: usize,
    pub entries: Vec<AuditEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AuditShowResult {
    pub schema_version: u32,
    pub journal_path: PathBuf,
    pub entry: AuditEntry,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AuditExportResult {
    pub schema_version: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub out: Option<PathBuf>,
    pub count: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub entries: Option<Vec<AuditEntry>>,
}

pub fn audit_journal_path(paths: &ConfigPaths) -> PathBuf {
    paths.audit_dir.join("audit.jsonl")
}

pub fn parse_timestamp_filter(value: &str) -> Result<u64> {
    if let Ok(number) = value.parse::<u64>() {
        return Ok(number);
    }

    let parsed = DateTime::parse_from_rfc3339(value).map_err(|source| GhlError::Validation {
        message: format!("invalid datetime `{value}`: {source}"),
    })?;
    Ok(parsed.timestamp_millis().max(0) as u64)
}

pub fn append_audit_entry(paths: &ConfigPaths, input: AuditEntryInput) -> Result<AuditEntry> {
    fs::create_dir_all(&paths.audit_dir).map_err(|source| GhlError::FileWrite {
        path: paths.audit_dir.clone(),
        source,
    })?;

    let entry = AuditEntry {
        schema_version: 1,
        id: new_audit_id(),
        timestamp_unix_ms: now_unix_ms(),
        profile: input.profile,
        company_id: input.company_id,
        location_id: input.location_id,
        command: input.command,
        action_class: input.action_class,
        dry_run: input.dry_run,
        policy_flags: input.policy_flags,
        resource: input.resource,
        request_summary: redact_json(&input.request_summary),
        upstream: input.upstream,
        result: redact_result(input.result),
        error: input.error.map(|error| redact_string(&error)),
    };

    append_jsonl(&audit_journal_path(paths), &entry)?;
    Ok(entry)
}

pub fn list_audit_entries(
    paths: &ConfigPaths,
    options: AuditListOptions,
) -> Result<AuditListResult> {
    let journal_path = audit_journal_path(paths);
    let mut entries = read_audit_entries(&journal_path)?
        .into_iter()
        .filter(|entry| audit_matches(entry, &options))
        .collect::<Vec<_>>();
    entries.sort_by(|left, right| right.timestamp_unix_ms.cmp(&left.timestamp_unix_ms));
    if let Some(limit) = options.limit {
        entries.truncate(limit);
    }

    Ok(AuditListResult {
        schema_version: 1,
        journal_path,
        count: entries.len(),
        entries,
    })
}

pub fn show_audit_entry(paths: &ConfigPaths, entry_id: &str) -> Result<AuditShowResult> {
    let journal_path = audit_journal_path(paths);
    let entry = read_audit_entries(&journal_path)?
        .into_iter()
        .find(|entry| entry.id == entry_id)
        .ok_or_else(|| GhlError::Validation {
            message: format!("audit entry `{entry_id}` was not found"),
        })?;

    Ok(AuditShowResult {
        schema_version: 1,
        journal_path,
        entry,
    })
}

pub fn export_audit_entries(
    paths: &ConfigPaths,
    options: AuditListOptions,
    out: Option<&Path>,
) -> Result<AuditExportResult> {
    let entries = list_audit_entries(paths, options)?.entries;
    if let Some(out) = out {
        write_private_json(out, &entries)?;
        Ok(AuditExportResult {
            schema_version: 1,
            out: Some(out.to_path_buf()),
            count: entries.len(),
            entries: None,
        })
    } else {
        Ok(AuditExportResult {
            schema_version: 1,
            out: None,
            count: entries.len(),
            entries: Some(entries),
        })
    }
}

fn read_audit_entries(path: &Path) -> Result<Vec<AuditEntry>> {
    if !path.exists() {
        return Ok(Vec::new());
    }

    let file = File::open(path).map_err(|source| GhlError::FileRead {
        path: path.to_path_buf(),
        source,
    })?;
    let reader = BufReader::new(file);
    let mut entries = Vec::new();
    for line in reader.lines() {
        let line = line.map_err(|source| GhlError::FileRead {
            path: path.to_path_buf(),
            source,
        })?;
        if line.trim().is_empty() {
            continue;
        }
        let entry =
            serde_json::from_str::<AuditEntry>(&line).map_err(|source| GhlError::ParseJson {
                path: path.display().to_string(),
                source,
            })?;
        entries.push(entry);
    }
    Ok(entries)
}

fn audit_matches(entry: &AuditEntry, options: &AuditListOptions) -> bool {
    if let Some(from) = options.from_unix_ms
        && entry.timestamp_unix_ms < from
    {
        return false;
    }
    if let Some(to) = options.to_unix_ms
        && entry.timestamp_unix_ms > to
    {
        return false;
    }
    if let Some(action) = &options.action
        && entry.action_class != *action
        && entry.command != *action
    {
        return false;
    }
    if let Some(resource) = &options.resource {
        let matches = entry.resource.as_ref().is_some_and(|entry_resource| {
            entry_resource.id.as_ref() == Some(resource)
                || entry_resource.resource_type == *resource
        });
        if !matches {
            return false;
        }
    }
    true
}

fn redact_result(result: AuditResultSummary) -> AuditResultSummary {
    AuditResultSummary {
        status: result.status,
        resource_id: result.resource_id,
        message: result.message.map(|message| redact_string(&message)),
    }
}

fn redact_string(value: &str) -> String {
    let redacted = redact_json(&json!({ "value": value }));
    redacted["value"]
        .as_str()
        .unwrap_or("[REDACTED]")
        .to_owned()
}

fn append_jsonl<T: Serialize>(path: &Path, value: &T) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|source| GhlError::FileWrite {
            path: parent.to_path_buf(),
            source,
        })?;
    }
    let mut file = private_append_options()
        .create(true)
        .append(true)
        .open(path)
        .map_err(|source| GhlError::FileWrite {
            path: path.to_path_buf(),
            source,
        })?;
    serde_json::to_writer(&mut file, value)?;
    file.write_all(b"\n")
        .map_err(|source| GhlError::FileWrite {
            path: path.to_path_buf(),
            source,
        })?;
    Ok(())
}

pub(crate) fn write_private_json<T: Serialize>(path: &Path, value: &T) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|source| GhlError::FileWrite {
            path: parent.to_path_buf(),
            source,
        })?;
    }
    let mut file = private_write_options()
        .create(true)
        .truncate(true)
        .write(true)
        .open(path)
        .map_err(|source| GhlError::FileWrite {
            path: path.to_path_buf(),
            source,
        })?;
    serde_json::to_writer_pretty(&mut file, value)?;
    file.write_all(b"\n")
        .map_err(|source| GhlError::FileWrite {
            path: path.to_path_buf(),
            source,
        })?;
    Ok(())
}

fn private_append_options() -> OpenOptions {
    let mut options = OpenOptions::new();
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.mode(0o600);
    }
    options
}

fn private_write_options() -> OpenOptions {
    let mut options = OpenOptions::new();
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.mode(0o600);
    }
    options
}

fn new_audit_id() -> String {
    let count = AUDIT_COUNTER.fetch_add(1, Ordering::Relaxed);
    format!("audit-{}-{count}", now_unix_ms())
}

fn now_unix_ms() -> u64 {
    let duration = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default();
    duration.as_millis().min(u128::from(u64::MAX)) as u64
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;
    use crate::config::resolve_paths;

    #[test]
    fn append_list_show_export_round_trip() {
        let temp = tempfile::tempdir().expect("tempdir");
        let paths = resolve_paths(Some(temp.path()));
        let entry = append_audit_entry(
            &paths,
            AuditEntryInput {
                profile: Some("default".to_owned()),
                company_id: Some("company_123".to_owned()),
                location_id: Some("loc_123".to_owned()),
                command: "contacts.create".to_owned(),
                action_class: "write".to_owned(),
                dry_run: false,
                policy_flags: vec!["confirmation_required".to_owned()],
                resource: Some(AuditResource {
                    resource_type: "contact".to_owned(),
                    id: Some("contact_123".to_owned()),
                }),
                request_summary: json!({ "email": "person@example.com" }),
                upstream: Some(AuditUpstreamSummary {
                    request_id: Some("req_123".to_owned()),
                    status_code: Some(201),
                    endpoint_key: Some("contacts.create".to_owned()),
                }),
                result: AuditResultSummary {
                    status: "success".to_owned(),
                    resource_id: Some("contact_123".to_owned()),
                    message: None,
                },
                error: None,
            },
        )
        .expect("append");

        let listed = list_audit_entries(
            &paths,
            AuditListOptions {
                action: Some("write".to_owned()),
                resource: Some("contact_123".to_owned()),
                limit: Some(10),
                ..AuditListOptions::default()
            },
        )
        .expect("list");
        assert_eq!(listed.entries, vec![entry.clone()]);

        let shown = show_audit_entry(&paths, &entry.id).expect("show");
        assert_eq!(shown.entry, entry);

        let out = temp.path().join("audit-export.json");
        let exported =
            export_audit_entries(&paths, AuditListOptions::default(), Some(&out)).expect("export");
        assert_eq!(exported.count, 1);
        assert!(out.exists());
    }

    #[test]
    fn audit_redacts_sensitive_fields_before_writing() {
        let temp = tempfile::tempdir().expect("tempdir");
        let paths = resolve_paths(Some(temp.path()));
        append_audit_entry(
            &paths,
            AuditEntryInput {
                profile: None,
                company_id: None,
                location_id: None,
                command: "auth.pit.add".to_owned(),
                action_class: "auth".to_owned(),
                dry_run: false,
                policy_flags: Vec::new(),
                resource: None,
                request_summary: json!({ "token": "pit-secret", "safe": "visible" }),
                upstream: None,
                result: AuditResultSummary {
                    status: "success".to_owned(),
                    resource_id: None,
                    message: Some("Bearer secret".to_owned()),
                },
                error: Some("Bearer secret".to_owned()),
            },
        )
        .expect("append");

        let rendered = fs::read_to_string(audit_journal_path(&paths)).expect("journal");
        assert!(!rendered.contains("pit-secret"));
        assert!(!rendered.contains("Bearer secret"));
        assert!(rendered.contains("visible"));
    }

    #[test]
    fn parses_rfc3339_and_unix_filters() {
        assert_eq!(parse_timestamp_filter("42").expect("number"), 42);
        assert_eq!(
            parse_timestamp_filter("1970-01-01T00:00:01Z").expect("rfc3339"),
            1000
        );
    }
}
