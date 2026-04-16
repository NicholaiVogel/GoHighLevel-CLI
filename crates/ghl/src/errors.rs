use std::path::PathBuf;

use serde::{Deserialize, Serialize};
use thiserror::Error;

pub type Result<T> = std::result::Result<T, GhlError>;

#[derive(Debug, Error)]
pub enum GhlError {
    #[error("{message}")]
    Validation { message: String },

    #[error("failed to read {path}: {source}")]
    FileRead {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },

    #[error("failed to write {path}: {source}")]
    FileWrite {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },

    #[error("failed to parse {path}: {source}")]
    ParseJson {
        path: String,
        #[source]
        source: serde_json::Error,
    },

    #[error("failed to serialize JSON response: {0}")]
    SerializeJson(#[from] serde_json::Error),

    #[error("endpoint `{key}` is not present in the bundled endpoint manifest")]
    EndpointNotFound { key: String },

    #[error("standard error code `{code}` is not defined")]
    ErrorCodeNotFound { code: String },

    #[error("offline mode blocked command `{command}` because it requires network access")]
    OfflineBlocked { command: String },

    #[error("profile `{profile}` does not exist")]
    ProfileNotFound { profile: String },

    #[error("credential `{credential_ref}` does not exist")]
    CredentialNotFound { credential_ref: String },

    #[error(
        "PIT token input is required; pass --token-stdin, --token-env <NAME>, or --token <TOKEN>"
    )]
    MissingTokenInput,

    #[error("pass only one PIT token source: --token-stdin, --token-env, or --token")]
    ConflictingTokenInput,

    #[error("destructive command requires confirmation; pass --yes to continue")]
    ConfirmationRequired,

    #[error("{message}")]
    AmbiguousContext { context: String, message: String },

    #[error("network request failed: {message}")]
    Network { message: String },
}

impl GhlError {
    pub fn code(&self) -> &'static str {
        match self {
            Self::Validation { .. } => "validation_error",
            Self::FileRead { .. } | Self::FileWrite { .. } => "file_io_error",
            Self::ParseJson { .. } => "parse_error",
            Self::SerializeJson(_) => "general_error",
            Self::EndpointNotFound { .. } | Self::ErrorCodeNotFound { .. } => "validation_error",
            Self::OfflineBlocked { .. } => "offline_blocked",
            Self::ProfileNotFound { .. }
            | Self::CredentialNotFound { .. }
            | Self::MissingTokenInput
            | Self::ConflictingTokenInput => "validation_error",
            Self::ConfirmationRequired => "confirmation_required",
            Self::AmbiguousContext { .. } => "ambiguous_context",
            Self::Network { .. } => "network_error",
        }
    }

    pub fn exit_code(&self) -> i32 {
        match self.code() {
            "validation_error" => 2,
            "file_io_error" => 9,
            "parse_error" => 8,
            "offline_blocked" => 17,
            "confirmation_required" => 15,
            "network_error" => 5,
            "ambiguous_context" => 2,
            _ => 1,
        }
    }

    pub fn hint(&self) -> Option<&'static str> {
        match self {
            Self::EndpointNotFound { .. } => {
                Some("Run `ghl endpoints list` to inspect bundled endpoint keys.")
            }
            Self::ErrorCodeNotFound { .. } => {
                Some("Run `ghl errors list` to inspect standard error codes.")
            }
            Self::OfflineBlocked { .. } => Some("Remove `--offline` or use a local-only command."),
            Self::ProfileNotFound { .. } => {
                Some("Run `ghl profiles list` to inspect configured profiles.")
            }
            Self::CredentialNotFound { .. } => {
                Some("Run `ghl auth pit list-local` to inspect local PIT credential references.")
            }
            Self::MissingTokenInput => {
                Some("Prefer `--token-stdin` so the token does not land in shell history.")
            }
            Self::ConflictingTokenInput => Some("Choose exactly one token source."),
            Self::ConfirmationRequired => {
                Some("Pass `--yes` only after reviewing the local change.")
            }
            Self::AmbiguousContext { context, .. } if context == "company" => {
                Some("Pass `--company <id>` or store a profile company id.")
            }
            Self::AmbiguousContext { context, .. } if context == "location" => {
                Some("Pass `--location <id>` or store a profile location id.")
            }
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ErrorDefinition {
    pub code: &'static str,
    pub exit_code: i32,
    pub title: &'static str,
    pub description: &'static str,
}

pub const STANDARD_ERRORS: &[ErrorDefinition] = &[
    ErrorDefinition {
        code: "general_error",
        exit_code: 1,
        title: "General error",
        description: "Unexpected local failure that does not fit a more specific category.",
    },
    ErrorDefinition {
        code: "validation_error",
        exit_code: 2,
        title: "CLI validation error",
        description: "Invalid command, flag, argument, local state, or unsupported Phase 0 request.",
    },
    ErrorDefinition {
        code: "authentication_error",
        exit_code: 3,
        title: "Authentication error",
        description: "Credentials are missing, expired, malformed, or rejected by GHL.",
    },
    ErrorDefinition {
        code: "authorization_error",
        exit_code: 4,
        title: "Authorization error",
        description: "The selected credential is valid but lacks permission for the requested operation.",
    },
    ErrorDefinition {
        code: "network_error",
        exit_code: 5,
        title: "Network error",
        description: "DNS, TLS, Cloudflare, connectivity, or transport failure.",
    },
    ErrorDefinition {
        code: "ghl_api_error",
        exit_code: 6,
        title: "GHL API error",
        description: "GHL returned a structured upstream API error.",
    },
    ErrorDefinition {
        code: "auth_class_unavailable",
        exit_code: 7,
        title: "Required auth class unavailable",
        description: "The command requires a token class that is not configured for the selected profile.",
    },
    ErrorDefinition {
        code: "parse_error",
        exit_code: 8,
        title: "Parse error",
        description: "Malformed JSON, HTML, CSV, fixture, or file input.",
    },
    ErrorDefinition {
        code: "file_io_error",
        exit_code: 9,
        title: "Local file I/O error",
        description: "The CLI could not read or write a required local file.",
    },
    ErrorDefinition {
        code: "timeout",
        exit_code: 10,
        title: "Timeout",
        description: "A local or remote operation exceeded its configured timeout.",
    },
    ErrorDefinition {
        code: "rate_limit_exceeded",
        exit_code: 11,
        title: "Rate limit exceeded",
        description: "GHL rate limiting stopped the operation and waiting was not permitted.",
    },
    ErrorDefinition {
        code: "policy_denied",
        exit_code: 12,
        title: "Profile policy denied",
        description: "Local profile policy refused the requested action before mutation.",
    },
    ErrorDefinition {
        code: "capability_unavailable",
        exit_code: 13,
        title: "Capability unavailable",
        description: "The requested GHL capability, plan feature, or endpoint is unavailable.",
    },
    ErrorDefinition {
        code: "schema_validation_failed",
        exit_code: 14,
        title: "Schema validation failed",
        description: "Structured input did not match the command schema.",
    },
    ErrorDefinition {
        code: "confirmation_required",
        exit_code: 15,
        title: "Confirmation required",
        description: "A destructive or sensitive command requires explicit confirmation.",
    },
    ErrorDefinition {
        code: "lock_unavailable",
        exit_code: 16,
        title: "Local lock unavailable",
        description: "A local config, credential, audit, or idempotency lock could not be acquired.",
    },
    ErrorDefinition {
        code: "offline_blocked",
        exit_code: 17,
        title: "Offline mode blocked command",
        description: "Offline mode refused a command that would require network access.",
    },
    ErrorDefinition {
        code: "ambiguous_context",
        exit_code: 2,
        title: "Ambiguous context",
        description: "Company or location context is missing or ambiguous.",
    },
];

pub fn error_definitions() -> &'static [ErrorDefinition] {
    STANDARD_ERRORS
}

pub fn find_error_definition(code: &str) -> Option<&'static ErrorDefinition> {
    STANDARD_ERRORS
        .iter()
        .find(|definition| definition.code == code)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validation_error_maps_to_exit_code_2() {
        let error = GhlError::Validation {
            message: "bad input".to_owned(),
        };

        assert_eq!(error.code(), "validation_error");
        assert_eq!(error.exit_code(), 2);
    }

    #[test]
    fn standard_error_codes_are_unique() {
        let mut codes = STANDARD_ERRORS
            .iter()
            .map(|definition| definition.code)
            .collect::<Vec<_>>();
        codes.sort_unstable();
        codes.dedup();

        assert_eq!(codes.len(), STANDARD_ERRORS.len());
    }

    #[test]
    fn standard_exit_codes_match_spec_range() {
        let exit_codes = STANDARD_ERRORS
            .iter()
            .map(|definition| definition.exit_code)
            .collect::<Vec<_>>();

        assert!(exit_codes.contains(&1));
        assert!(exit_codes.contains(&17));
        assert!(exit_codes.iter().all(|code| (1..=17).contains(code)));
    }
}
