use serde::{Deserialize, Serialize};

use crate::config::ConfigPaths;
use crate::credentials::{
    CredentialStore, RedactedCredential, credential_ref, load_credentials, save_credentials,
};
use crate::errors::{GhlError, Result};
use crate::profiles::{Profile, load_profiles, save_profiles};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PitAddResult {
    pub profile: String,
    pub credential_ref: String,
    pub location_id: Option<String>,
    pub company_id: Option<String>,
    pub auth_class: String,
    pub backend: String,
    pub validated: bool,
    pub warning: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct LocalPitList {
    pub backend: String,
    pub credentials: Vec<RedactedCredential>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PitRemoveResult {
    pub credential_ref: String,
    pub removed: bool,
    pub profiles_updated: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AuthStatus {
    pub profile: String,
    pub location_id: Option<String>,
    pub company_id: Option<String>,
    pub auth: AuthClasses,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AuthClasses {
    pub pit: AuthClassStatus,
    pub session: AuthClassStatus,
    pub firebase: AuthClassStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AuthClassStatus {
    pub available: bool,
    pub credential_ref: Option<String>,
    pub backend: Option<String>,
    pub validated_at_unix: Option<u64>,
    pub expires_at: Option<String>,
    pub hint: Option<String>,
}

pub fn add_pit(
    paths: &ConfigPaths,
    profile_name: &str,
    token: String,
    location_id: Option<String>,
    company_id: Option<String>,
    set_default: bool,
) -> Result<PitAddResult> {
    if token.trim().is_empty() {
        return Err(GhlError::MissingTokenInput);
    }

    let mut profiles = load_profiles(paths)?;
    let credential_ref = credential_ref(profile_name, "pit");
    {
        let profile = profiles.get_or_create(profile_name);
        profile.credential_refs.pit = Some(credential_ref.clone());
        if location_id.is_some() {
            profile.location_id = location_id.clone();
        }
        if company_id.is_some() {
            profile.company_id = company_id.clone();
        }
    }
    if set_default || profiles.default_profile.is_none() {
        profiles.default_profile = Some(profile_name.to_owned());
    }

    let mut credentials = load_credentials(paths)?;
    credentials.upsert_secret(&credential_ref, profile_name, "pit", token);

    save_profiles(paths, &profiles)?;
    save_credentials(paths, &credentials)?;

    Ok(PitAddResult {
        profile: profile_name.to_owned(),
        credential_ref,
        location_id,
        company_id,
        auth_class: "pit".to_owned(),
        backend: credentials.backend,
        validated: false,
        warning: "Stored locally but not validated yet; Phase 1 does not perform live GHL reads."
            .to_owned(),
    })
}

pub fn list_local_pits(paths: &ConfigPaths) -> Result<LocalPitList> {
    let credentials = load_credentials(paths)?;
    let redacted = credentials
        .credentials
        .values()
        .filter(|credential| credential.auth_class == "pit")
        .map(|credential| credential.redacted())
        .collect();

    Ok(LocalPitList {
        backend: credentials.backend,
        credentials: redacted,
    })
}

pub fn remove_local_pit(paths: &ConfigPaths, credential_id: &str) -> Result<PitRemoveResult> {
    let mut credentials = load_credentials(paths)?;
    let removed = credentials.remove(credential_id).is_some();
    if !removed {
        return Err(GhlError::CredentialNotFound {
            credential_ref: credential_id.to_owned(),
        });
    }

    let mut profiles = load_profiles(paths)?;
    let mut profiles_updated = Vec::new();
    for profile in profiles.profiles.values_mut() {
        if profile.credential_refs.pit.as_deref() == Some(credential_id) {
            profile.credential_refs.pit = None;
            profiles_updated.push(profile.name.clone());
        }
    }

    save_credentials(paths, &credentials)?;
    save_profiles(paths, &profiles)?;

    Ok(PitRemoveResult {
        credential_ref: credential_id.to_owned(),
        removed,
        profiles_updated,
    })
}

pub fn auth_status(paths: &ConfigPaths, requested_profile: Option<&str>) -> Result<AuthStatus> {
    let profiles = load_profiles(paths)?;
    let profile_name = profiles.selected_name(requested_profile).to_owned();
    let profile = profiles.get_required(&profile_name)?;
    let credentials = load_credentials(paths)?;

    Ok(status_for_profile(profile, &credentials))
}

fn status_for_profile(profile: &Profile, credentials: &CredentialStore) -> AuthStatus {
    AuthStatus {
        profile: profile.name.clone(),
        location_id: profile.location_id.clone(),
        company_id: profile.company_id.clone(),
        auth: AuthClasses {
            pit: status_for_ref(
                profile.credential_refs.pit.as_deref(),
                credentials,
                "Run `ghl auth pit add --token-stdin --location <id>`.",
            ),
            session: status_for_ref(
                profile.credential_refs.session.as_deref(),
                credentials,
                "Session login is not implemented in this slice.",
            ),
            firebase: status_for_ref(
                profile.credential_refs.firebase.as_deref(),
                credentials,
                "Firebase exchange is not implemented in this slice.",
            ),
        },
    }
}

fn status_for_ref(
    credential_ref: Option<&str>,
    credentials: &CredentialStore,
    missing_hint: &str,
) -> AuthClassStatus {
    match credential_ref {
        Some(reference) if credentials.get(reference).is_some() => AuthClassStatus {
            available: true,
            credential_ref: Some(reference.to_owned()),
            backend: Some(credentials.backend.clone()),
            validated_at_unix: credentials
                .get(reference)
                .and_then(|credential| credential.validated_at_unix),
            expires_at: None,
            hint: None,
        },
        Some(reference) => AuthClassStatus {
            available: false,
            credential_ref: Some(reference.to_owned()),
            backend: Some(credentials.backend.clone()),
            validated_at_unix: None,
            expires_at: None,
            hint: Some(
                "Credential reference exists, but the backing secret is missing.".to_owned(),
            ),
        },
        None => AuthClassStatus {
            available: false,
            credential_ref: None,
            backend: Some(credentials.backend.clone()),
            validated_at_unix: None,
            expires_at: None,
            hint: Some(missing_hint.to_owned()),
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::resolve_paths;

    #[test]
    fn add_pit_creates_profile_and_redacted_list() {
        let temp = tempfile::tempdir().expect("tempdir");
        let paths = resolve_paths(Some(temp.path()));

        add_pit(
            &paths,
            "default",
            "pit-secret-1234".to_owned(),
            Some("loc_123".to_owned()),
            None,
            true,
        )
        .expect("add pit");

        let profiles = load_profiles(&paths).expect("profiles");
        assert_eq!(profiles.default_profile.as_deref(), Some("default"));
        assert_eq!(
            profiles.profiles["default"].credential_refs.pit.as_deref(),
            Some("pit:default")
        );

        let pits = list_local_pits(&paths).expect("pits");
        assert_eq!(pits.credentials.len(), 1);
        assert_eq!(pits.credentials[0].secret_preview, "********1234");
    }

    #[test]
    fn status_reports_pit_available_after_add() {
        let temp = tempfile::tempdir().expect("tempdir");
        let paths = resolve_paths(Some(temp.path()));
        add_pit(
            &paths,
            "default",
            "pit-secret".to_owned(),
            Some("loc_123".to_owned()),
            Some("cmp_123".to_owned()),
            true,
        )
        .expect("add pit");

        let status = auth_status(&paths, Some("default")).expect("status");

        assert!(status.auth.pit.available);
        assert!(!status.auth.session.available);
        assert_eq!(status.location_id.as_deref(), Some("loc_123"));
    }
}
