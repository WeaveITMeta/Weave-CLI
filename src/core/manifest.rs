// =============================================================================
// Manifest - Parsing and representing the weave.manifest.toml file
// =============================================================================
//
// Table of Contents:
// - WeaveManifest: Root manifest structure
// - TemplateInfo: Template metadata (name, version, minimum CLI version)
// - ManifestEntry: A single selectable option with its directory mappings
// - ManifestCategory: A named group of entries (platforms, backends, etc.)
// - Parsing functions: Load manifest from file path or string
// =============================================================================

use anyhow::{Context, Result};
use serde::Deserialize;
use std::collections::HashMap;
use std::path::Path;

/// Root structure of the weave.manifest.toml file.
/// This file lives inside the Weave-Template repository and defines
/// all available options the CLI wizard presents to the user.
#[derive(Debug, Deserialize, Clone)]
pub struct WeaveManifest {
    /// Template metadata
    pub template: TemplateInfo,

    /// Platform stack options (Nexpo web, Nexpo mobile, Taurte web, etc.)
    #[serde(default)]
    pub platforms: HashMap<String, ManifestEntry>,

    /// Backend API language options (TypeScript, Rust, Go, etc.)
    #[serde(default)]
    pub backends: HashMap<String, ManifestEntry>,

    /// Authentication provider options (Supabase, Auth0, Firebase, None)
    #[serde(default)]
    pub auth: HashMap<String, ManifestEntry>,

    /// Database options (Supabase/PostgreSQL, MongoDB, etc.)
    #[serde(default)]
    pub database: HashMap<String, ManifestEntry>,

    /// Cloud infrastructure provider options (AWS, GCP, Azure, etc.)
    #[serde(default)]
    pub cloud: HashMap<String, ManifestEntry>,

    /// Microservice options (payments, notifications, etc.)
    #[serde(default)]
    pub microservices: HashMap<String, ManifestEntry>,

    /// Infrastructure/DevOps tool options (Docker, Terraform, Redis, etc.)
    #[serde(default)]
    pub infrastructure: HashMap<String, ManifestEntry>,

    /// Extra feature options (email, i18n, Stripe, SEO, CI/CD)
    #[serde(default)]
    pub extras: HashMap<String, ManifestEntry>,
}

/// Metadata about the template itself
#[derive(Debug, Deserialize, Clone)]
pub struct TemplateInfo {
    /// Human-readable template name
    pub name: String,

    /// Template version (should match the git tag)
    pub version: String,

    /// Minimum CLI version required to use this template
    #[serde(default)]
    pub minimum_cli_version: Option<String>,
}

/// A single selectable option within a category.
/// Maps the user's choice to the directories to keep, environment variables
/// to include, Docker services to enable, and Terraform modules to include.
#[derive(Debug, Deserialize, Clone)]
pub struct ManifestEntry {
    /// Display label shown in the wizard UI
    pub label: String,

    /// Longer description shown in the preview panel
    #[serde(default)]
    pub description: Option<String>,

    /// Directory paths to keep when this option is selected.
    /// Supports glob patterns (for example, "packages/shared-nexpo/*").
    #[serde(default)]
    pub keep: Vec<String>,

    /// Additional Terraform directories to keep
    #[serde(default)]
    pub terraform_keep: Vec<String>,

    /// Environment variables required by this option
    #[serde(default)]
    pub env_vars: Vec<String>,

    /// Docker Compose service names to include
    #[serde(default)]
    pub docker_services: Vec<String>,

    /// Other option keys that this option depends on
    #[serde(default)]
    pub requires: Vec<String>,

    /// Other option keys that conflict with this option
    #[serde(default)]
    pub conflicts_with: Vec<String>,

    /// Bun packages to add to the root package.json dependencies
    #[serde(default)]
    pub dependencies: Vec<String>,

    /// Bun packages to add to the root package.json devDependencies
    #[serde(default)]
    pub dev_dependencies: Vec<String>,
}

/// Parse a manifest from a file path
pub fn parse_manifest_file(path: &Path) -> Result<WeaveManifest> {
    let content = std::fs::read_to_string(path)
        .with_context(|| format!("Failed to read manifest file: {}", path.display()))?;
    parse_manifest_string(&content)
}

/// Parse a manifest from a TOML string
pub fn parse_manifest_string(content: &str) -> Result<WeaveManifest> {
    let manifest: WeaveManifest =
        toml::from_str(content).context("Failed to parse weave.manifest.toml")?;
    Ok(manifest)
}

impl WeaveManifest {
    /// Get all entries for a given category name as a vector of (key, entry) pairs,
    /// sorted alphabetically by key for consistent display order.
    pub fn get_category_entries(&self, category: &str) -> Vec<(&String, &ManifestEntry)> {
        let map = match category {
            "platforms" => &self.platforms,
            "backends" => &self.backends,
            "auth" => &self.auth,
            "database" => &self.database,
            "cloud" => &self.cloud,
            "microservices" => &self.microservices,
            "infrastructure" => &self.infrastructure,
            "extras" => &self.extras,
            _ => return Vec::new(),
        };
        let mut entries: Vec<_> = map.iter().collect();
        entries.sort_by_key(|(key, _)| key.to_owned());
        entries
    }

    /// Get all category names in wizard display order
    pub fn category_order() -> &'static [&'static str] {
        &[
            "platforms",
            "backends",
            "auth",
            "database",
            "cloud",
            "microservices",
            "infrastructure",
            "extras",
        ]
    }

    /// Collect all directories to keep based on a set of selected keys per category
    pub fn collect_keep_paths(
        &self,
        selections: &HashMap<String, Vec<String>>,
    ) -> Vec<String> {
        let mut paths = Vec::new();

        for (category, selected_keys) in selections {
            let map = match category.as_str() {
                "platforms" => &self.platforms,
                "backends" => &self.backends,
                "auth" => &self.auth,
                "database" => &self.database,
                "cloud" => &self.cloud,
                "microservices" => &self.microservices,
                "infrastructure" => &self.infrastructure,
                "extras" => &self.extras,
                _ => continue,
            };

            for key in selected_keys {
                if let Some(entry) = map.get(key) {
                    paths.extend(entry.keep.clone());
                    paths.extend(entry.terraform_keep.clone());
                }
            }
        }

        paths
    }

    /// Collect all required environment variables based on selections
    pub fn collect_env_vars(
        &self,
        selections: &HashMap<String, Vec<String>>,
    ) -> Vec<String> {
        let mut env_vars = Vec::new();

        for (category, selected_keys) in selections {
            let map = match category.as_str() {
                "platforms" => &self.platforms,
                "backends" => &self.backends,
                "auth" => &self.auth,
                "database" => &self.database,
                "cloud" => &self.cloud,
                "microservices" => &self.microservices,
                "infrastructure" => &self.infrastructure,
                "extras" => &self.extras,
                _ => continue,
            };

            for key in selected_keys {
                if let Some(entry) = map.get(key) {
                    env_vars.extend(entry.env_vars.clone());
                }
            }
        }

        env_vars.sort();
        env_vars.dedup();
        env_vars
    }

    /// Collect all Docker service names based on selections
    pub fn collect_docker_services(
        &self,
        selections: &HashMap<String, Vec<String>>,
    ) -> Vec<String> {
        let mut services = Vec::new();

        for (category, selected_keys) in selections {
            let map = match category.as_str() {
                "platforms" => &self.platforms,
                "backends" => &self.backends,
                "auth" => &self.auth,
                "database" => &self.database,
                "cloud" => &self.cloud,
                "microservices" => &self.microservices,
                "infrastructure" => &self.infrastructure,
                "extras" => &self.extras,
                _ => continue,
            };

            for key in selected_keys {
                if let Some(entry) = map.get(key) {
                    services.extend(entry.docker_services.clone());
                }
            }
        }

        services.sort();
        services.dedup();
        services
    }
}
