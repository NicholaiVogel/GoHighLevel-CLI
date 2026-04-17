use serde::{Deserialize, Serialize};

use crate::errors::{GhlError, Result};

const BUNDLED_ENDPOINTS_JSON: &str = include_str!("../../../data/endpoints.json");

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct EndpointManifest {
    pub schema_version: u32,
    pub source: String,
    pub generated_from: Vec<String>,
    pub status: String,
    pub endpoints: Vec<EndpointDefinition>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct EndpointDefinition {
    pub endpoint_key: String,
    pub surface: String,
    pub method: String,
    pub path_template: String,
    pub auth_classes: Vec<String>,
    pub source_refs: Vec<String>,
    pub risk: String,
    pub status: String,
    pub phase: String,
    pub command_keys: Vec<String>,
    pub response_schema: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct EndpointCoverage {
    pub schema_version: u32,
    pub status: String,
    pub endpoint_count: usize,
    pub command_mapped_count: usize,
    pub implemented_count: usize,
    pub note: String,
}

pub fn bundled_manifest() -> Result<EndpointManifest> {
    serde_json::from_str(BUNDLED_ENDPOINTS_JSON).map_err(|source| GhlError::ParseJson {
        path: "data/endpoints.json".to_owned(),
        source,
    })
}

pub fn endpoint_coverage(manifest: &EndpointManifest) -> EndpointCoverage {
    EndpointCoverage {
        schema_version: manifest.schema_version,
        status: manifest.status.clone(),
        endpoint_count: manifest.endpoints.len(),
        command_mapped_count: manifest
            .endpoints
            .iter()
            .filter(|endpoint| !endpoint.command_keys.is_empty())
            .count(),
        implemented_count: manifest
            .endpoints
            .iter()
            .filter(|endpoint| endpoint.status == "implemented")
            .count(),
        note: "Endpoint records are added slice by slice as commands become safe and testable."
            .to_owned(),
    }
}

pub fn find_endpoint<'a>(
    manifest: &'a EndpointManifest,
    endpoint_key: &str,
) -> Option<&'a EndpointDefinition> {
    manifest
        .endpoints
        .iter()
        .find(|endpoint| endpoint.endpoint_key == endpoint_key)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bundled_manifest_loads() {
        let manifest = bundled_manifest().expect("manifest");

        assert_eq!(manifest.schema_version, 1);
        assert_eq!(manifest.status, "scaffold");
    }

    #[test]
    fn bundled_manifest_counts_implemented_read_endpoints() {
        let manifest = bundled_manifest().expect("manifest");
        let coverage = endpoint_coverage(&manifest);

        assert_eq!(coverage.endpoint_count, 14);
        assert_eq!(coverage.command_mapped_count, 14);
        assert_eq!(coverage.implemented_count, 14);
    }
}
