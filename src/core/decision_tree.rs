// =============================================================================
// Decision Tree - Validation and dependency resolution for selections
// =============================================================================
//
// Table of Contents:
// - validate_selections: Check for conflicts and missing dependencies
// - resolve_dependencies: Auto-enable required options
// - ValidationResult: Errors and warnings from validation
// =============================================================================

use super::manifest::WeaveManifest;
use super::selections::UserSelections;
use std::collections::HashMap;

/// Result of validating user selections against manifest rules
#[derive(Debug, Default)]
pub struct ValidationResult {
    /// Hard errors that prevent scaffolding
    pub errors: Vec<String>,

    /// Warnings that the user should acknowledge
    pub warnings: Vec<String>,

    /// Options that were auto-enabled due to dependency resolution
    pub auto_enabled: Vec<(String, String)>,
}

impl ValidationResult {
    /// Returns true if there are no blocking errors
    pub fn is_valid(&self) -> bool {
        self.errors.is_empty()
    }
}

/// Validate user selections against the manifest's dependency and conflict rules
pub fn validate_selections(
    manifest: &WeaveManifest,
    selections: &UserSelections,
) -> ValidationResult {
    let mut result = ValidationResult::default();

    // Check that required categories have a selection
    if selections.get("platforms").map_or(true, |v| v.is_empty()) {
        result
            .errors
            .push("A platform stack must be selected.".to_string());
    }

    if selections.get("backends").map_or(true, |v| v.is_empty()) {
        result
            .errors
            .push("A backend language must be selected.".to_string());
    }

    // Check for conflicts within each category
    for category in WeaveManifest::category_order() {
        let entries_map = match *category {
            "platforms" => &manifest.platforms,
            "backends" => &manifest.backends,
            "auth" => &manifest.auth,
            "database" => &manifest.database,
            "cloud" => &manifest.cloud,
            "microservices" => &manifest.microservices,
            "infrastructure" => &manifest.infrastructure,
            "extras" => &manifest.extras,
            _ => continue,
        };

        if let Some(selected_keys) = selections.get(category) {
            for key in selected_keys {
                if let Some(entry) = entries_map.get(key) {
                    // Check conflicts
                    for conflict in &entry.conflicts_with {
                        if selections.is_selected(category, conflict) {
                            result.errors.push(format!(
                                "Conflict: '{}' and '{}' cannot both be selected in {}.",
                                entry.label,
                                conflict,
                                category
                            ));
                        }
                    }

                    // Check cross-category dependencies
                    for requirement in &entry.requires {
                        let found = is_requirement_satisfied(selections, requirement);
                        if !found {
                            result.warnings.push(format!(
                                "'{}' requires '{}' which is not selected. It will be auto-enabled.",
                                entry.label, requirement
                            ));
                        }
                    }
                }
            }
        }
    }

    result
}

/// Resolve dependencies by auto-enabling required options.
/// Returns a new selections object with dependencies added.
pub fn resolve_dependencies(
    manifest: &WeaveManifest,
    selections: &mut UserSelections,
) -> Vec<(String, String)> {
    let mut auto_enabled: Vec<(String, String)> = Vec::new();

    // Iterate through all selected options and check their requirements
    let selections_snapshot: HashMap<String, Vec<String>> = selections.selections.clone();

    for (category, selected_keys) in &selections_snapshot {
        let entries_map = match category.as_str() {
            "platforms" => &manifest.platforms,
            "backends" => &manifest.backends,
            "auth" => &manifest.auth,
            "database" => &manifest.database,
            "cloud" => &manifest.cloud,
            "microservices" => &manifest.microservices,
            "infrastructure" => &manifest.infrastructure,
            "extras" => &manifest.extras,
            _ => continue,
        };

        for key in selected_keys {
            if let Some(entry) = entries_map.get(key) {
                for requirement in &entry.requires {
                    // Try to find which category the requirement belongs to
                    if let Some((required_category, required_key)) =
                        find_entry_category(manifest, requirement)
                    {
                        if !selections.is_selected(&required_category, &required_key) {
                            // Auto-enable the required option
                            let current = selections
                                .selections
                                .entry(required_category.clone())
                                .or_default();
                            current.push(required_key.clone());
                            auto_enabled.push((required_category, required_key));
                        }
                    }
                }
            }
        }
    }

    auto_enabled
}

/// Check if a requirement string is satisfied by the current selections.
/// Requirements can be in the format "category.key" or just "key" (searched across all categories).
fn is_requirement_satisfied(selections: &UserSelections, requirement: &str) -> bool {
    if let Some((category, key)) = requirement.split_once('.') {
        selections.is_selected(category, key)
    } else {
        // Search across all categories for the key
        for category in WeaveManifest::category_order() {
            if selections.is_selected(category, requirement) {
                return true;
            }
        }
        false
    }
}

/// Find which category a given key belongs to in the manifest.
/// Supports "category.key" format or plain "key" (first match wins).
fn find_entry_category(manifest: &WeaveManifest, requirement: &str) -> Option<(String, String)> {
    if let Some((category, key)) = requirement.split_once('.') {
        return Some((category.to_string(), key.to_string()));
    }

    // Search all categories for the key
    let categories: Vec<(&str, &std::collections::HashMap<String, _>)> = vec![
        ("platforms", &manifest.platforms),
        ("backends", &manifest.backends),
        ("auth", &manifest.auth),
        ("database", &manifest.database),
        ("cloud", &manifest.cloud),
        ("microservices", &manifest.microservices),
        ("infrastructure", &manifest.infrastructure),
        ("extras", &manifest.extras),
    ];

    for (category, map) in categories {
        if map.contains_key(requirement) {
            return Some((category.to_string(), requirement.to_string()));
        }
    }

    None
}
