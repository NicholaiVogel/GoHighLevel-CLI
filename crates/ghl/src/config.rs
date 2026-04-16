use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::errors::Result;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ConfigPaths {
    pub source: String,
    pub config_dir: PathBuf,
    pub config_file: PathBuf,
    pub profiles_file: PathBuf,
    pub credentials_file: PathBuf,
    pub data_dir: PathBuf,
    pub cache_dir: PathBuf,
    pub audit_dir: PathBuf,
    pub jobs_dir: PathBuf,
    pub locks_dir: PathBuf,
    pub bundled_endpoint_manifest: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CliConfig {
    pub schema_version: u32,
    pub paths: ConfigPaths,
    pub default_profile: Option<String>,
    pub profiles: Vec<String>,
    pub note: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ConfigDoctor {
    pub ok: bool,
    pub paths: Vec<PathCheck>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PathCheck {
    pub name: String,
    pub path: PathBuf,
    pub exists: bool,
    pub readable: bool,
    pub writable: bool,
}

pub fn resolve_paths_from_env(config_dir_flag: Option<&Path>) -> Result<ConfigPaths> {
    if let Some(config_dir) = config_dir_flag {
        return Ok(paths_from_override(config_dir, "flag"));
    }

    if let Some(config_dir) = std::env::var_os("GHL_CLI_CONFIG_DIR") {
        return Ok(paths_from_override(Path::new(&config_dir), "env"));
    }

    Ok(resolve_paths(None))
}

pub fn resolve_paths(config_dir_override: Option<&Path>) -> ConfigPaths {
    if let Some(config_dir) = config_dir_override {
        return paths_from_override(config_dir, "override");
    }

    let config_dir = dirs::config_dir()
        .unwrap_or_else(fallback_config_dir)
        .join("ghl-cli");
    let data_dir = dirs::data_local_dir()
        .unwrap_or_else(fallback_data_dir)
        .join("ghl-cli");
    let cache_dir = dirs::cache_dir()
        .unwrap_or_else(fallback_cache_dir)
        .join("ghl-cli");

    paths_from_parts("platform", config_dir, data_dir, cache_dir)
}

pub fn redacted_config(paths: ConfigPaths) -> CliConfig {
    CliConfig {
        schema_version: 1,
        paths,
        default_profile: None,
        profiles: Vec::new(),
        note: "Phase 0 has not implemented profile persistence yet.".to_owned(),
    }
}

pub fn config_doctor(paths: &ConfigPaths) -> ConfigDoctor {
    let checks = vec![
        check_path("config_dir", &paths.config_dir),
        check_path("config_file", &paths.config_file),
        check_path("profiles_file", &paths.profiles_file),
        check_path("credentials_file", &paths.credentials_file),
        check_path("data_dir", &paths.data_dir),
        check_path("cache_dir", &paths.cache_dir),
        check_path("audit_dir", &paths.audit_dir),
        check_path("jobs_dir", &paths.jobs_dir),
        check_path("locks_dir", &paths.locks_dir),
    ];
    let ok = checks.iter().all(|check| check.readable || check.writable);

    ConfigDoctor { ok, paths: checks }
}

fn paths_from_override(config_dir: &Path, source: &str) -> ConfigPaths {
    let root = config_dir.to_path_buf();
    paths_from_parts(source, root.clone(), root.join("data"), root.join("cache"))
}

fn paths_from_parts(
    source: impl Into<String>,
    config_dir: PathBuf,
    data_dir: PathBuf,
    cache_dir: PathBuf,
) -> ConfigPaths {
    ConfigPaths {
        source: source.into(),
        config_file: config_dir.join("config.json"),
        profiles_file: config_dir.join("profiles.json"),
        credentials_file: config_dir.join("credentials.json"),
        audit_dir: data_dir.join("audit"),
        jobs_dir: data_dir.join("jobs"),
        locks_dir: data_dir.join("locks"),
        bundled_endpoint_manifest: "data/endpoints.json".to_owned(),
        config_dir,
        data_dir,
        cache_dir,
    }
}

fn check_path(name: &str, path: &Path) -> PathCheck {
    let exists = path.exists();
    let readable = std::fs::metadata(path).is_ok();
    let writable = if exists {
        std::fs::metadata(path)
            .map(|metadata| !metadata.permissions().readonly())
            .unwrap_or(false)
    } else {
        path.parent()
            .and_then(|parent| std::fs::metadata(parent).ok())
            .map(|metadata| !metadata.permissions().readonly())
            .unwrap_or(false)
    };

    PathCheck {
        name: name.to_owned(),
        path: path.to_path_buf(),
        exists,
        readable,
        writable,
    }
}

fn fallback_config_dir() -> PathBuf {
    dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join(".config")
}

fn fallback_data_dir() -> PathBuf {
    dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join(".local")
        .join("share")
}

fn fallback_cache_dir() -> PathBuf {
    dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join(".cache")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn explicit_override_controls_all_local_paths() {
        let root = PathBuf::from("/tmp/ghl-cli-test");
        let paths = resolve_paths(Some(&root));

        assert_eq!(paths.source, "override");
        assert_eq!(paths.config_dir, root);
        assert_eq!(
            paths.config_file,
            PathBuf::from("/tmp/ghl-cli-test/config.json")
        );
        assert_eq!(paths.cache_dir, PathBuf::from("/tmp/ghl-cli-test/cache"));
        assert_eq!(
            paths.audit_dir,
            PathBuf::from("/tmp/ghl-cli-test/data/audit")
        );
    }

    #[test]
    fn doctor_reports_existing_temp_root() {
        let temp = tempfile::tempdir().expect("tempdir");
        let paths = resolve_paths(Some(temp.path()));
        let doctor = config_doctor(&paths);

        assert!(
            doctor
                .paths
                .iter()
                .any(|check| check.name == "config_dir" && check.exists)
        );
        assert!(doctor.paths.iter().any(|check| check.name == "cache_dir"));
    }

    #[test]
    fn redacted_config_has_no_profiles_in_phase_zero() {
        let paths = resolve_paths(Some(Path::new("/tmp/ghl-cli-test")));
        let config = redacted_config(paths);

        assert_eq!(config.schema_version, 1);
        assert!(config.default_profile.is_none());
        assert!(config.profiles.is_empty());
    }
}
