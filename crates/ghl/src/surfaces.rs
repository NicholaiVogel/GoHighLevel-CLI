use serde::{Deserialize, Serialize};

use crate::profiles::Profile;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Surface {
    Services,
    Backend,
}

impl Surface {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Services => "services",
            Self::Backend => "backend",
        }
    }

    pub fn base_url(self, profile: &Profile) -> &str {
        match self {
            Self::Services => &profile.base_urls.services,
            Self::Backend => &profile.base_urls.backend,
        }
    }
}

impl std::str::FromStr for Surface {
    type Err = crate::errors::GhlError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "services" => Ok(Self::Services),
            "backend" => Ok(Self::Backend),
            _ => Err(crate::errors::GhlError::Validation {
                message: format!("unsupported surface `{value}`; expected services or backend"),
            }),
        }
    }
}
