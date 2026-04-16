use serde::{Deserialize, Serialize};

use crate::errors::{GhlError, Result};
use crate::profiles::{Profile, load_profiles};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ResolvedContext {
    pub profile: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub company_id: Option<ResolvedContextValue>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub location_id: Option<ResolvedContextValue>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ResolvedContextValue {
    pub value: String,
    pub source: ContextSource,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ContextSource {
    Override,
    Profile,
}

impl ResolvedContext {
    pub fn require_company_id(&self) -> Result<&str> {
        self.company_id
            .as_ref()
            .map(|value| value.value.as_str())
            .ok_or_else(|| GhlError::AmbiguousContext {
                context: "company".to_owned(),
                message: "company context is required; pass --company or set a profile company id"
                    .to_owned(),
            })
    }

    pub fn require_location_id(&self) -> Result<&str> {
        self.location_id
            .as_ref()
            .map(|value| value.value.as_str())
            .ok_or_else(|| GhlError::AmbiguousContext {
                context: "location".to_owned(),
                message:
                    "location context is required; pass --location or set a profile location id"
                        .to_owned(),
            })
    }
}

pub fn resolve_context(
    paths: &crate::ConfigPaths,
    profile_name: Option<&str>,
    location_override: Option<&str>,
    company_override: Option<&str>,
) -> Result<ResolvedContext> {
    let profiles = load_profiles(paths)?;
    let selected = profiles.selected_name(profile_name).to_owned();
    let profile = profiles.get_required(&selected)?;

    Ok(resolve_context_for_profile(
        profile,
        location_override,
        company_override,
    ))
}

pub fn resolve_context_for_dry_run(
    paths: &crate::ConfigPaths,
    profile_name: Option<&str>,
    location_override: Option<&str>,
    company_override: Option<&str>,
) -> Result<ResolvedContext> {
    let profiles = load_profiles(paths)?;
    let selected = profiles.selected_name(profile_name).to_owned();
    if let Some(profile) = profiles.profiles.get(&selected) {
        return Ok(resolve_context_for_profile(
            profile,
            location_override,
            company_override,
        ));
    }

    if location_override.is_some() || company_override.is_some() {
        return Ok(ResolvedContext {
            profile: selected,
            company_id: resolve_value(company_override, None),
            location_id: resolve_value(location_override, None),
        });
    }

    Err(GhlError::ProfileNotFound { profile: selected })
}

pub fn resolve_context_for_profile(
    profile: &Profile,
    location_override: Option<&str>,
    company_override: Option<&str>,
) -> ResolvedContext {
    ResolvedContext {
        profile: profile.name.clone(),
        company_id: resolve_value(company_override, profile.company_id.as_deref()),
        location_id: resolve_value(location_override, profile.location_id.as_deref()),
    }
}

fn resolve_value(
    override_value: Option<&str>,
    profile_value: Option<&str>,
) -> Option<ResolvedContextValue> {
    override_value
        .filter(|value| !value.trim().is_empty())
        .map(|value| ResolvedContextValue {
            value: value.to_owned(),
            source: ContextSource::Override,
        })
        .or_else(|| {
            profile_value
                .filter(|value| !value.trim().is_empty())
                .map(|value| ResolvedContextValue {
                    value: value.to_owned(),
                    source: ContextSource::Profile,
                })
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn override_wins_over_profile_context() {
        let mut profile = Profile::new("default");
        profile.company_id = Some("company_profile".to_owned());
        profile.location_id = Some("loc_profile".to_owned());

        let context =
            resolve_context_for_profile(&profile, Some("loc_override"), Some("company_override"));

        assert_eq!(
            context.company_id.unwrap(),
            ResolvedContextValue {
                value: "company_override".to_owned(),
                source: ContextSource::Override,
            }
        );
        assert_eq!(
            context.location_id.unwrap(),
            ResolvedContextValue {
                value: "loc_override".to_owned(),
                source: ContextSource::Override,
            }
        );
    }

    #[test]
    fn profile_context_is_used_when_override_missing() {
        let mut profile = Profile::new("default");
        profile.company_id = Some("company_profile".to_owned());

        let context = resolve_context_for_profile(&profile, None, None);

        assert_eq!(context.require_company_id().unwrap(), "company_profile");
    }

    #[test]
    fn required_company_context_returns_ambiguous_context() {
        let profile = Profile::new("default");
        let context = resolve_context_for_profile(&profile, None, None);

        let error = context.require_company_id().expect_err("missing company");

        assert_eq!(error.code(), "ambiguous_context");
        assert_eq!(error.exit_code(), 2);
    }
}
