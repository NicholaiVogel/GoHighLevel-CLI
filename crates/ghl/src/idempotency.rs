use std::collections::BTreeMap;
use std::fs::{self, File, OpenOptions};
use std::hash::{Hash, Hasher};
use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::config::ConfigPaths;
use crate::errors::{GhlError, Result};
use crate::redaction::redact_json;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct IdempotencyRecord {
    pub schema_version: u32,
    pub key: String,
    pub scoped_key: String,
    pub profile: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub location_id: Option<String>,
    pub command: String,
    pub request_hash: String,
    pub status: IdempotencyStatus,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub resource_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub audit_entry_id: Option<String>,
    pub created_at_unix_ms: u64,
    pub updated_at_unix_ms: u64,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum IdempotencyStatus {
    InProgress,
    Succeeded,
    Failed,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct IdempotencyPut {
    pub key: String,
    pub profile: String,
    pub location_id: Option<String>,
    pub command: String,
    pub request_hash: String,
    pub status: IdempotencyStatus,
    pub resource_id: Option<String>,
    pub audit_entry_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct IdempotencyCheck {
    pub key: String,
    pub scoped_key: String,
    pub state: IdempotencyCheckState,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub existing: Option<IdempotencyRecord>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum IdempotencyCheckState {
    Available,
    Replay,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct IdempotencyListResult {
    pub schema_version: u32,
    pub store_path: PathBuf,
    pub count: usize,
    pub records: Vec<IdempotencyRecord>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct IdempotencyShowResult {
    pub schema_version: u32,
    pub store_path: PathBuf,
    pub record: IdempotencyRecord,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct IdempotencyClearResult {
    pub schema_version: u32,
    pub store_path: PathBuf,
    pub key: String,
    pub removed: bool,
    pub remaining_count: usize,
}

pub fn idempotency_store_path(paths: &ConfigPaths) -> PathBuf {
    paths.data_dir.join("idempotency").join("idempotency.jsonl")
}

pub fn stable_request_hash(value: &Value) -> Result<String> {
    let redacted = redact_json(value);
    let bytes = serde_json::to_vec(&redacted)?;
    let mut hasher = StableHasher::default();
    bytes.hash(&mut hasher);
    Ok(format!("fnv1a64:{:016x}", hasher.finish()))
}

pub fn scoped_idempotency_key(
    profile: &str,
    location_id: Option<&str>,
    command: &str,
    key: &str,
) -> String {
    format!(
        "{}:{}:{}:{}",
        profile,
        location_id.unwrap_or("global"),
        command,
        key
    )
}

pub fn check_idempotency_key(
    paths: &ConfigPaths,
    profile: &str,
    location_id: Option<&str>,
    command: &str,
    key: &str,
    request_hash: &str,
) -> Result<IdempotencyCheck> {
    let scoped_key = scoped_idempotency_key(profile, location_id, command, key);
    let existing = compacted_records(&idempotency_store_path(paths))?
        .into_iter()
        .find(|record| record.scoped_key == scoped_key);

    if let Some(existing) = existing {
        if existing.request_hash != request_hash {
            return Err(GhlError::Validation {
                message: format!(
                    "idempotency key `{key}` is already used for a different request hash"
                ),
            });
        }
        return Ok(IdempotencyCheck {
            key: key.to_owned(),
            scoped_key,
            state: IdempotencyCheckState::Replay,
            existing: Some(existing),
        });
    }

    Ok(IdempotencyCheck {
        key: key.to_owned(),
        scoped_key,
        state: IdempotencyCheckState::Available,
        existing: None,
    })
}

pub fn record_idempotency_key(
    paths: &ConfigPaths,
    put: IdempotencyPut,
) -> Result<IdempotencyRecord> {
    let now = now_unix_ms();
    let scoped_key = scoped_idempotency_key(
        &put.profile,
        put.location_id.as_deref(),
        &put.command,
        &put.key,
    );
    let previous = compacted_records(&idempotency_store_path(paths))?
        .into_iter()
        .find(|record| record.scoped_key == scoped_key);
    if let Some(previous) = &previous
        && previous.request_hash != put.request_hash
    {
        return Err(GhlError::Validation {
            message: format!(
                "idempotency key `{}` is already used for a different request hash",
                put.key
            ),
        });
    }

    let record = IdempotencyRecord {
        schema_version: 1,
        key: put.key,
        scoped_key,
        profile: put.profile,
        location_id: put.location_id,
        command: put.command,
        request_hash: put.request_hash,
        status: put.status,
        resource_id: put.resource_id,
        audit_entry_id: put.audit_entry_id,
        created_at_unix_ms: previous
            .as_ref()
            .map(|record| record.created_at_unix_ms)
            .unwrap_or(now),
        updated_at_unix_ms: now,
    };

    append_record(&idempotency_store_path(paths), &record)?;
    Ok(record)
}

pub fn list_idempotency_records(paths: &ConfigPaths) -> Result<IdempotencyListResult> {
    let store_path = idempotency_store_path(paths);
    let mut records = compacted_records(&store_path)?;
    records.sort_by(|left, right| right.updated_at_unix_ms.cmp(&left.updated_at_unix_ms));
    Ok(IdempotencyListResult {
        schema_version: 1,
        store_path,
        count: records.len(),
        records,
    })
}

pub fn show_idempotency_record(paths: &ConfigPaths, key: &str) -> Result<IdempotencyShowResult> {
    let store_path = idempotency_store_path(paths);
    let record = compacted_records(&store_path)?
        .into_iter()
        .find(|record| record.key == key || record.scoped_key == key)
        .ok_or_else(|| GhlError::Validation {
            message: format!("idempotency key `{key}` was not found"),
        })?;
    Ok(IdempotencyShowResult {
        schema_version: 1,
        store_path,
        record,
    })
}

pub fn clear_idempotency_record(paths: &ConfigPaths, key: &str) -> Result<IdempotencyClearResult> {
    let store_path = idempotency_store_path(paths);
    let mut records = compacted_records(&store_path)?;
    let before = records.len();
    records.retain(|record| record.key != key && record.scoped_key != key);
    let removed = records.len() != before;
    rewrite_records(&store_path, &records)?;

    Ok(IdempotencyClearResult {
        schema_version: 1,
        store_path,
        key: key.to_owned(),
        removed,
        remaining_count: records.len(),
    })
}

fn compacted_records(path: &Path) -> Result<Vec<IdempotencyRecord>> {
    let mut by_scope = BTreeMap::<String, IdempotencyRecord>::new();
    for record in read_records(path)? {
        by_scope.insert(record.scoped_key.clone(), record);
    }
    Ok(by_scope.into_values().collect())
}

fn read_records(path: &Path) -> Result<Vec<IdempotencyRecord>> {
    if !path.exists() {
        return Ok(Vec::new());
    }

    let file = File::open(path).map_err(|source| GhlError::FileRead {
        path: path.to_path_buf(),
        source,
    })?;
    let reader = BufReader::new(file);
    let mut records = Vec::new();
    for line in reader.lines() {
        let line = line.map_err(|source| GhlError::FileRead {
            path: path.to_path_buf(),
            source,
        })?;
        if line.trim().is_empty() {
            continue;
        }
        if let Ok(record) = serde_json::from_str::<IdempotencyRecord>(&line) {
            records.push(record);
            continue;
        }
        let values = serde_json::from_str::<Vec<IdempotencyRecord>>(&line).map_err(|source| {
            GhlError::ParseJson {
                path: path.display().to_string(),
                source,
            }
        })?;
        records.extend(values);
    }
    Ok(records)
}

fn rewrite_records(path: &Path, records: &[IdempotencyRecord]) -> Result<()> {
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
    for record in records {
        serde_json::to_writer(&mut file, record)?;
        file.write_all(b"\n")
            .map_err(|source| GhlError::FileWrite {
                path: path.to_path_buf(),
                source,
            })?;
    }
    Ok(())
}

fn append_record(path: &Path, record: &IdempotencyRecord) -> Result<()> {
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
    serde_json::to_writer(&mut file, record)?;
    file.write_all(b"\n")
        .map_err(|source| GhlError::FileWrite {
            path: path.to_path_buf(),
            source,
        })?;
    Ok(())
}

fn private_append_options() -> OpenOptions {
    private_options()
}

fn private_write_options() -> OpenOptions {
    private_options()
}

fn private_options() -> OpenOptions {
    let mut options = OpenOptions::new();
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.mode(0o600);
    }
    options
}

fn now_unix_ms() -> u64 {
    let duration = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default();
    duration.as_millis().min(u128::from(u64::MAX)) as u64
}

#[derive(Default)]
struct StableHasher(u64);

impl Hasher for StableHasher {
    fn finish(&self) -> u64 {
        self.0
    }

    fn write(&mut self, bytes: &[u8]) {
        let mut hash = if self.0 == 0 {
            0xcbf29ce484222325
        } else {
            self.0
        };
        for byte in bytes {
            hash ^= u64::from(*byte);
            hash = hash.wrapping_mul(0x100000001b3);
        }
        self.0 = hash;
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;
    use crate::config::resolve_paths;

    #[test]
    fn record_list_show_clear_round_trip() {
        let temp = tempfile::tempdir().expect("tempdir");
        let paths = resolve_paths(Some(temp.path()));
        let request_hash = stable_request_hash(&json!({ "name": "Jane" })).expect("hash");
        let record = record_idempotency_key(
            &paths,
            IdempotencyPut {
                key: "create-contact-1".to_owned(),
                profile: "default".to_owned(),
                location_id: Some("loc_123".to_owned()),
                command: "contacts.create".to_owned(),
                request_hash: request_hash.clone(),
                status: IdempotencyStatus::Succeeded,
                resource_id: Some("contact_123".to_owned()),
                audit_entry_id: Some("audit_123".to_owned()),
            },
        )
        .expect("record");

        let check = check_idempotency_key(
            &paths,
            "default",
            Some("loc_123"),
            "contacts.create",
            "create-contact-1",
            &request_hash,
        )
        .expect("check");
        assert_eq!(check.state, IdempotencyCheckState::Replay);
        assert_eq!(check.existing, Some(record.clone()));

        let listed = list_idempotency_records(&paths).expect("list");
        assert_eq!(listed.records, vec![record.clone()]);
        let shown = show_idempotency_record(&paths, "create-contact-1").expect("show");
        assert_eq!(shown.record, record);

        record_idempotency_key(
            &paths,
            IdempotencyPut {
                key: "create-contact-2".to_owned(),
                profile: "default".to_owned(),
                location_id: Some("loc_123".to_owned()),
                command: "contacts.create".to_owned(),
                request_hash: stable_request_hash(&json!({ "name": "June" })).expect("hash"),
                status: IdempotencyStatus::Succeeded,
                resource_id: Some("contact_456".to_owned()),
                audit_entry_id: Some("audit_456".to_owned()),
            },
        )
        .expect("record second");

        let cleared = clear_idempotency_record(&paths, "create-contact-1").expect("clear");
        assert!(cleared.removed);
        let remaining = list_idempotency_records(&paths).expect("list");
        assert_eq!(remaining.count, 1);
        assert_eq!(remaining.records[0].key, "create-contact-2");
    }

    #[test]
    fn reused_key_with_different_hash_fails() {
        let temp = tempfile::tempdir().expect("tempdir");
        let paths = resolve_paths(Some(temp.path()));
        record_idempotency_key(
            &paths,
            IdempotencyPut {
                key: "same-key".to_owned(),
                profile: "default".to_owned(),
                location_id: None,
                command: "contacts.create".to_owned(),
                request_hash: stable_request_hash(&json!({ "a": 1 })).expect("hash"),
                status: IdempotencyStatus::Succeeded,
                resource_id: None,
                audit_entry_id: None,
            },
        )
        .expect("record");

        let error = check_idempotency_key(
            &paths,
            "default",
            None,
            "contacts.create",
            "same-key",
            &stable_request_hash(&json!({ "a": 2 })).expect("hash"),
        )
        .expect_err("conflict");
        assert_eq!(error.code(), "validation_error");
    }

    #[test]
    fn request_hash_redacts_secret_like_values() {
        let one =
            stable_request_hash(&json!({ "token": "pit-secret-one", "safe": "x" })).expect("hash");
        let two =
            stable_request_hash(&json!({ "token": "pit-secret-two", "safe": "x" })).expect("hash");
        assert_eq!(one, two);
    }
}
