use std::collections::BTreeMap;
use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::config::{ConfigPaths, redacted_config};
use crate::errors::{GhlError, Result};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ProfilesFile {
    pub schema_version: u32,
    pub default_profile: Option<String>,
    pub profiles: BTreeMap<String, Profile>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Profile {
    pub name: String,
    pub base_urls: BaseUrls,
    pub company_id: Option<String>,
    pub location_id: Option<String>,
    pub user_id: Option<String>,
    pub credential_refs: CredentialRefs,
    pub policy: ProfilePolicy,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct BaseUrls {
    pub services: String,
    pub backend: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct CredentialRefs {
    pub pit: Option<String>,
    pub session: Option<String>,
    pub firebase: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ProfilePolicy {
    pub agent_mode: bool,
    pub default_dry_run: bool,
    pub allow_destructive: bool,
    pub allow_messaging: bool,
    pub allow_payment_actions: bool,
    pub allow_public_links: bool,
    pub allow_workflow_publish: bool,
    pub allow_phone_purchase: bool,
    pub allow_private_integration_token_create: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ProfileList {
    pub default_profile: Option<String>,
    pub profiles: Vec<ProfileSummary>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ProfileSummary {
    pub name: String,
    pub company_id: Option<String>,
    pub location_id: Option<String>,
    pub has_pit: bool,
    pub has_session: bool,
    pub has_firebase: bool,
    pub is_default: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ProfileDefaultResult {
    pub profile: String,
    pub default_profile: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ProfileLocationResult {
    pub profile: String,
    pub location_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ProfileCompanyResult {
    pub profile: String,
    pub company_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct ProfilePolicyPatch {
    pub agent_mode: Option<bool>,
    pub default_dry_run: Option<bool>,
    pub allow_destructive: Option<bool>,
    pub allow_messaging: Option<bool>,
    pub allow_payment_actions: Option<bool>,
    pub allow_public_links: Option<bool>,
    pub allow_workflow_publish: Option<bool>,
    pub allow_phone_purchase: Option<bool>,
    pub allow_private_integration_token_create: Option<bool>,
}

impl ProfilesFile {
    pub fn empty() -> Self {
        Self {
            schema_version: 1,
            default_profile: None,
            profiles: BTreeMap::new(),
        }
    }

    pub fn list(&self) -> ProfileList {
        ProfileList {
            default_profile: self.default_profile.clone(),
            profiles: self
                .profiles
                .values()
                .map(|profile| ProfileSummary {
                    name: profile.name.clone(),
                    company_id: profile.company_id.clone(),
                    location_id: profile.location_id.clone(),
                    has_pit: profile.credential_refs.pit.is_some(),
                    has_session: profile.credential_refs.session.is_some(),
                    has_firebase: profile.credential_refs.firebase.is_some(),
                    is_default: self.default_profile.as_ref() == Some(&profile.name),
                })
                .collect(),
        }
    }

    pub fn selected_name<'a>(&'a self, requested: Option<&'a str>) -> &'a str {
        requested
            .or(self.default_profile.as_deref())
            .unwrap_or("default")
    }

    pub fn get_required(&self, name: &str) -> Result<&Profile> {
        self.profiles
            .get(name)
            .ok_or_else(|| GhlError::ProfileNotFound {
                profile: name.to_owned(),
            })
    }

    pub fn get_required_mut(&mut self, name: &str) -> Result<&mut Profile> {
        self.profiles
            .get_mut(name)
            .ok_or_else(|| GhlError::ProfileNotFound {
                profile: name.to_owned(),
            })
    }

    pub fn get_or_create(&mut self, name: &str) -> &mut Profile {
        self.profiles
            .entry(name.to_owned())
            .or_insert_with(|| Profile::new(name))
    }
}

impl Profile {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_owned(),
            base_urls: BaseUrls::default(),
            company_id: None,
            location_id: None,
            user_id: None,
            credential_refs: CredentialRefs::default(),
            policy: ProfilePolicy::default(),
        }
    }
}

impl Default for BaseUrls {
    fn default() -> Self {
        Self {
            services: "https://services.leadconnectorhq.com".to_owned(),
            backend: "https://backend.leadconnectorhq.com".to_owned(),
        }
    }
}

impl Default for ProfilePolicy {
    fn default() -> Self {
        Self {
            agent_mode: true,
            default_dry_run: true,
            allow_destructive: false,
            allow_messaging: false,
            allow_payment_actions: false,
            allow_public_links: false,
            allow_workflow_publish: false,
            allow_phone_purchase: false,
            allow_private_integration_token_create: false,
        }
    }
}

impl ProfilePolicy {
    pub fn apply_patch(&mut self, patch: &ProfilePolicyPatch) -> bool {
        let mut changed = false;
        macro_rules! apply {
            ($field:ident) => {
                if let Some(value) = patch.$field {
                    if self.$field != value {
                        self.$field = value;
                        changed = true;
                    }
                }
            };
        }

        apply!(agent_mode);
        apply!(default_dry_run);
        apply!(allow_destructive);
        apply!(allow_messaging);
        apply!(allow_payment_actions);
        apply!(allow_public_links);
        apply!(allow_workflow_publish);
        apply!(allow_phone_purchase);
        apply!(allow_private_integration_token_create);

        changed
    }
}

pub fn load_profiles(paths: &ConfigPaths) -> Result<ProfilesFile> {
    load_profiles_from_path(&paths.profiles_file)
}

pub fn load_profiles_from_path(path: &Path) -> Result<ProfilesFile> {
    if !path.exists() {
        return Ok(ProfilesFile::empty());
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

pub fn save_profiles(paths: &ConfigPaths, profiles: &ProfilesFile) -> Result<()> {
    save_profiles_to_path(&paths.profiles_file, profiles)
}

pub fn save_profiles_to_path(path: &Path, profiles: &ProfilesFile) -> Result<()> {
    write_json_pretty(path, profiles)
}

pub fn set_default_profile(paths: &ConfigPaths, name: &str) -> Result<ProfileDefaultResult> {
    let mut profiles = load_profiles(paths)?;
    profiles.get_required(name)?;
    profiles.default_profile = Some(name.to_owned());
    save_profiles(paths, &profiles)?;

    Ok(ProfileDefaultResult {
        profile: name.to_owned(),
        default_profile: name.to_owned(),
    })
}

pub fn set_default_location(
    paths: &ConfigPaths,
    profile_name: &str,
    location_id: &str,
) -> Result<ProfileLocationResult> {
    let mut profiles = load_profiles(paths)?;
    let profile = profiles.get_required_mut(profile_name)?;
    profile.location_id = Some(location_id.to_owned());
    save_profiles(paths, &profiles)?;

    Ok(ProfileLocationResult {
        profile: profile_name.to_owned(),
        location_id: location_id.to_owned(),
    })
}

pub fn set_default_company(
    paths: &ConfigPaths,
    profile_name: &str,
    company_id: &str,
) -> Result<ProfileCompanyResult> {
    let mut profiles = load_profiles(paths)?;
    let profile = profiles.get_required_mut(profile_name)?;
    profile.company_id = Some(company_id.to_owned());
    save_profiles(paths, &profiles)?;

    Ok(ProfileCompanyResult {
        profile: profile_name.to_owned(),
        company_id: company_id.to_owned(),
    })
}

pub fn redacted_config_with_profiles(paths: ConfigPaths) -> Result<crate::config::CliConfig> {
    let profiles = load_profiles(&paths)?;
    let mut config = redacted_config(paths);
    config.default_profile = profiles.default_profile;
    config.profiles = profiles.profiles.keys().cloned().collect();
    config.note = "Secrets are stored separately and never included in config output.".to_owned();
    Ok(config)
}

pub fn write_json_pretty<T>(path: &Path, value: &T) -> Result<()>
where
    T: Serialize,
{
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|source| GhlError::FileWrite {
            path: parent.to_path_buf(),
            source,
        })?;
    }

    let temp_path = path.with_extension("tmp");
    let body = serde_json::to_vec_pretty(value)?;
    std::fs::write(&temp_path, body).map_err(|source| GhlError::FileWrite {
        path: temp_path.clone(),
        source,
    })?;
    std::fs::rename(&temp_path, path).map_err(|source| GhlError::FileWrite {
        path: path.to_path_buf(),
        source,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::resolve_paths;

    #[test]
    fn missing_profiles_file_loads_empty_store() {
        let temp = tempfile::tempdir().expect("tempdir");
        let paths = resolve_paths(Some(temp.path()));
        let profiles = load_profiles(&paths).expect("profiles");

        assert_eq!(profiles.schema_version, 1);
        assert!(profiles.profiles.is_empty());
    }

    #[test]
    fn save_and_load_profile_round_trips() {
        let temp = tempfile::tempdir().expect("tempdir");
        let paths = resolve_paths(Some(temp.path()));
        let mut profiles = ProfilesFile::empty();
        let profile = profiles.get_or_create("default");
        profile.location_id = Some("loc_123".to_owned());
        profiles.default_profile = Some("default".to_owned());

        save_profiles(&paths, &profiles).expect("save");
        let loaded = load_profiles(&paths).expect("load");

        assert_eq!(loaded.default_profile.as_deref(), Some("default"));
        assert_eq!(
            loaded.profiles["default"].location_id.as_deref(),
            Some("loc_123")
        );
    }

    #[test]
    fn default_policy_is_agent_safe() {
        let policy = ProfilePolicy::default();

        assert!(policy.agent_mode);
        assert!(policy.default_dry_run);
        assert!(!policy.allow_destructive);
        assert!(!policy.allow_messaging);
        assert!(!policy.allow_payment_actions);
    }
}
