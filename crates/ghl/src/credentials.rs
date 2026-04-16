use std::collections::BTreeMap;
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};

use crate::config::ConfigPaths;
use crate::errors::{GhlError, Result};
use crate::profiles::write_json_pretty;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CredentialStore {
    pub schema_version: u32,
    pub backend: String,
    pub credentials: BTreeMap<String, StoredCredential>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct StoredCredential {
    pub id: String,
    pub profile: String,
    pub auth_class: String,
    pub secret: String,
    pub created_at_unix: u64,
    pub updated_at_unix: u64,
    #[serde(default)]
    pub validated_at_unix: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RedactedCredential {
    pub id: String,
    pub profile: String,
    pub auth_class: String,
    pub secret_preview: String,
    pub created_at_unix: u64,
    pub updated_at_unix: u64,
    #[serde(default)]
    pub validated_at_unix: Option<u64>,
}

impl CredentialStore {
    pub fn empty() -> Self {
        Self {
            schema_version: 1,
            backend: "file".to_owned(),
            credentials: BTreeMap::new(),
        }
    }

    pub fn upsert_secret(&mut self, id: &str, profile: &str, auth_class: &str, secret: String) {
        let now = now_unix();
        let created_at_unix = self
            .credentials
            .get(id)
            .map(|credential| credential.created_at_unix)
            .unwrap_or(now);
        self.credentials.insert(
            id.to_owned(),
            StoredCredential {
                id: id.to_owned(),
                profile: profile.to_owned(),
                auth_class: auth_class.to_owned(),
                secret,
                created_at_unix,
                updated_at_unix: now,
                validated_at_unix: self
                    .credentials
                    .get(id)
                    .and_then(|credential| credential.validated_at_unix),
            },
        );
    }

    pub fn remove(&mut self, id: &str) -> Option<StoredCredential> {
        self.credentials.remove(id)
    }

    pub fn get(&self, id: &str) -> Option<&StoredCredential> {
        self.credentials.get(id)
    }

    pub fn mark_validated(&mut self, id: &str) {
        let now = now_unix();
        if let Some(credential) = self.credentials.get_mut(id) {
            credential.validated_at_unix = Some(now);
            credential.updated_at_unix = now;
        }
    }
}

impl StoredCredential {
    pub fn redacted(&self) -> RedactedCredential {
        RedactedCredential {
            id: self.id.clone(),
            profile: self.profile.clone(),
            auth_class: self.auth_class.clone(),
            secret_preview: redacted_secret_preview(&self.secret),
            created_at_unix: self.created_at_unix,
            updated_at_unix: self.updated_at_unix,
            validated_at_unix: self.validated_at_unix,
        }
    }
}

pub fn credential_ref(profile: &str, auth_class: &str) -> String {
    format!("{auth_class}:{profile}")
}

pub fn load_credentials(paths: &ConfigPaths) -> Result<CredentialStore> {
    load_credentials_from_path(&paths.credentials_file)
}

pub fn load_credentials_from_path(path: &Path) -> Result<CredentialStore> {
    if !path.exists() {
        return Ok(CredentialStore::empty());
    }

    let bytes = std::fs::read(path).map_err(|source| GhlError::FileRead {
        path: path.to_path_buf(),
        source,
    })?;
    serde_json::from_slice(&bytes).map_err(|source| GhlError::ParseJson {
        path: path.display().to_string(),
        source,
    })
}

pub fn save_credentials(paths: &ConfigPaths, store: &CredentialStore) -> Result<()> {
    save_credentials_to_path(&paths.credentials_file, store)
}

pub fn save_credentials_to_path(path: &Path, store: &CredentialStore) -> Result<()> {
    write_json_pretty(path, store)?;
    set_owner_only_permissions(path)?;
    Ok(())
}

fn redacted_secret_preview(secret: &str) -> String {
    let tail: String = secret
        .chars()
        .rev()
        .take(4)
        .collect::<String>()
        .chars()
        .rev()
        .collect();
    if tail.is_empty() {
        "********".to_owned()
    } else {
        format!("********{tail}")
    }
}

fn now_unix() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

#[cfg(unix)]
fn set_owner_only_permissions(path: &Path) -> Result<()> {
    use std::os::unix::fs::PermissionsExt;

    let mut permissions = std::fs::metadata(path)
        .map_err(|source| GhlError::FileRead {
            path: path.to_path_buf(),
            source,
        })?
        .permissions();
    permissions.set_mode(0o600);
    std::fs::set_permissions(path, permissions).map_err(|source| GhlError::FileWrite {
        path: path.to_path_buf(),
        source,
    })
}

#[cfg(not(unix))]
fn set_owner_only_permissions(_path: &Path) -> Result<()> {
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::resolve_paths;

    #[test]
    fn credential_ref_is_stable() {
        assert_eq!(credential_ref("default", "pit"), "pit:default");
    }

    #[test]
    fn save_and_load_credentials_round_trips() {
        let temp = tempfile::tempdir().expect("tempdir");
        let paths = resolve_paths(Some(temp.path()));
        let mut store = CredentialStore::empty();
        store.upsert_secret("pit:default", "default", "pit", "pit-secret".to_owned());

        save_credentials(&paths, &store).expect("save");
        let loaded = load_credentials(&paths).expect("load");

        assert_eq!(loaded.get("pit:default").unwrap().secret, "pit-secret");
    }

    #[test]
    fn redacted_credential_keeps_only_tail() {
        let mut store = CredentialStore::empty();
        store.upsert_secret("pit:default", "default", "pit", "abcdef1234".to_owned());
        let redacted = store.get("pit:default").unwrap().redacted();

        assert_eq!(redacted.secret_preview, "********1234");
    }
}
