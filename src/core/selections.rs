// =============================================================================
// Selections - User selection state throughout the wizard flow
// =============================================================================
//
// Table of Contents:
// - UserSelections: Complete set of choices the user makes in the wizard
// - SelectionMode: Whether a category is single-select or multi-select
// - Serialization to/from weave.toml project config
// =============================================================================

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Complete set of user selections from the wizard.
/// This is serialized to a weave.toml file in the scaffolded project
/// for reproducibility (re-scaffold with same choices).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserSelections {
    /// Project name (directory name)
    pub project_name: String,

    /// Selected options per category.
    /// Key: category name (for example, "platforms", "backends")
    /// Value: list of selected option keys (single item for radio, multiple for checkbox)
    pub selections: HashMap<String, Vec<String>>,
}

/// Whether a wizard step allows one selection or multiple
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SelectionMode {
    /// User picks exactly one option (radio button behavior)
    Single,

    /// User picks zero or more options (checkbox behavior)
    Multi,

    /// User picks zero or one option (optional radio)
    OptionalSingle,
}

impl UserSelections {
    /// Create a new empty selections object with the given project name
    pub fn new(project_name: String) -> Self {
        Self {
            project_name,
            selections: HashMap::new(),
        }
    }

    /// Set a single selection for a category (replaces any previous selection)
    pub fn set_single(&mut self, category: &str, key: String) {
        self.selections.insert(category.to_string(), vec![key]);
    }

    /// Set multiple selections for a category (replaces any previous selections)
    pub fn set_multi(&mut self, category: &str, keys: Vec<String>) {
        self.selections.insert(category.to_string(), keys);
    }

    /// Get selected keys for a category
    pub fn get(&self, category: &str) -> Option<&Vec<String>> {
        self.selections.get(category)
    }

    /// Check if a specific key is selected in a category
    pub fn is_selected(&self, category: &str, key: &str) -> bool {
        self.selections
            .get(category)
            .map(|keys| keys.iter().any(|k| k == key))
            .unwrap_or(false)
    }

    /// Get the selection mode for each wizard category
    pub fn selection_mode_for(category: &str) -> SelectionMode {
        match category {
            // Single-select categories (pick exactly one)
            "platforms" => SelectionMode::Single,
            "backends" => SelectionMode::Single,

            // Optional single-select (pick zero or one)
            "auth" => SelectionMode::OptionalSingle,
            "database" => SelectionMode::OptionalSingle,
            "cloud" => SelectionMode::OptionalSingle,

            // Multi-select categories (pick zero or more)
            "microservices" => SelectionMode::Multi,
            "infrastructure" => SelectionMode::Multi,
            "extras" => SelectionMode::Multi,

            _ => SelectionMode::Multi,
        }
    }

    /// Serialize selections to a TOML string for saving as weave.toml
    pub fn to_toml_string(&self) -> Result<String, toml::ser::Error> {
        toml::to_string_pretty(self)
    }

    /// Deserialize selections from a TOML string
    pub fn from_toml_string(content: &str) -> Result<Self, toml::de::Error> {
        toml::from_str(content)
    }

    /// Get a human-readable summary of all selections for the summary screen
    pub fn summary_lines(&self) -> Vec<(String, String)> {
        let order = [
            "platforms",
            "backends",
            "auth",
            "database",
            "cloud",
            "microservices",
            "infrastructure",
            "extras",
        ];

        let mut lines = Vec::new();
        for category in &order {
            let display_name = category_display_name(category);
            let value = self
                .selections
                .get(*category)
                .map(|keys| {
                    if keys.is_empty() {
                        "None".to_string()
                    } else {
                        keys.join(", ")
                    }
                })
                .unwrap_or_else(|| "Not selected".to_string());
            lines.push((display_name, value));
        }
        lines
    }
}

/// Convert a category key to a human-readable display name
pub fn category_display_name(category: &str) -> String {
    match category {
        "platforms" => "Platform Stack".to_string(),
        "backends" => "Backend Language".to_string(),
        "auth" => "Authentication".to_string(),
        "database" => "Database".to_string(),
        "cloud" => "Cloud Provider".to_string(),
        "microservices" => "Microservices".to_string(),
        "infrastructure" => "Infrastructure".to_string(),
        "extras" => "Extras".to_string(),
        other => other.to_string(),
    }
}

/// Convert a category key to a short instruction for the wizard
pub fn category_instruction(category: &str) -> String {
    match category {
        "platforms" => "Choose your platform stack".to_string(),
        "backends" => "Choose your backend API language".to_string(),
        "auth" => "Choose an authentication provider (optional)".to_string(),
        "database" => "Choose a database (optional)".to_string(),
        "cloud" => "Choose a cloud provider (optional)".to_string(),
        "microservices" => "Select microservices to include".to_string(),
        "infrastructure" => "Select infrastructure tools to include".to_string(),
        "extras" => "Select additional features".to_string(),
        _ => String::new(),
    }
}
