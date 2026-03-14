// =============================================================================
// Generator - Generate configuration files based on user selections
// =============================================================================
//
// Table of Contents:
// - generate_package_json: Create root package.json with bun as package manager
// - generate_workspace_config: Create bun workspace configuration
// - generate_env_file: Create .env from required environment variables
// - generate_weave_toml: Save user selections for reproducibility
// - generate_docker_compose: Filter docker-compose.yml to selected services
// - generate_pnpm_to_bun: Convert pnpm workspace config to bun workspace
// - run_scaffold: Orchestrate the full scaffolding process
// =============================================================================

use crate::core::manifest::WeaveManifest;
use crate::core::selections::UserSelections;
use anyhow::{Context, Result};
use std::collections::HashSet;
use std::path::Path;

/// Orchestrate the full scaffolding process after template is copied and pruned.
/// This rewrites configuration files to match the user's selections and uses bun.
pub fn post_scaffold(
    project_dir: &Path,
    manifest: &WeaveManifest,
    selections: &UserSelections,
    skip_git: bool,
) -> Result<()> {
    // Step 1: Save user selections as weave.toml
    generate_weave_toml(project_dir, selections)?;

    // Step 2: Rewrite package.json to use bun and only include selected workspaces
    rewrite_package_json(project_dir, manifest, selections)?;

    // Step 3: Generate bun workspace configuration (replaces pnpm-workspace.yaml)
    generate_bun_workspace(project_dir, manifest, selections)?;

    // Step 4: Generate .env file with required environment variables
    generate_env_file(project_dir, manifest, selections)?;

    // Step 5: Filter docker-compose.yml to only include selected services
    filter_docker_compose(project_dir, manifest, selections)?;

    // Step 6: Clean up files that are not needed in the scaffolded project
    cleanup_artifacts(project_dir)?;

    // Step 7: Initialize git repository (unless skipped)
    if !skip_git {
        initialize_git(project_dir)?;
    }

    Ok(())
}

/// Save user selections as a weave.toml file in the project root for reproducibility.
/// Running `weave init --config weave.toml` will recreate the same project.
fn generate_weave_toml(project_dir: &Path, selections: &UserSelections) -> Result<()> {
    let toml_content = selections
        .to_toml_string()
        .context("Failed to serialize selections to TOML")?;

    let weave_toml_path = project_dir.join("weave.toml");
    std::fs::write(&weave_toml_path, toml_content)
        .context("Failed to write weave.toml")?;

    tracing::info!("Generated weave.toml");
    Ok(())
}

/// Rewrite the root package.json to use bun and only reference selected workspaces
fn rewrite_package_json(
    project_dir: &Path,
    manifest: &WeaveManifest,
    selections: &UserSelections,
) -> Result<()> {
    let package_json_path = project_dir.join("package.json");

    if !package_json_path.exists() {
        tracing::warn!("No package.json found in template, skipping rewrite");
        return Ok(());
    }

    let content = std::fs::read_to_string(&package_json_path)
        .context("Failed to read package.json")?;

    let mut package: serde_json::Value =
        serde_json::from_str(&content).context("Failed to parse package.json")?;

    // Update project name
    package["name"] = serde_json::Value::String(selections.project_name.clone());

    // Remove pnpm-specific fields
    package.as_object_mut().map(|obj| {
        obj.remove("packageManager");
        obj.remove("pnpm");
    });

    // Rebuild workspaces array based on selected keep paths
    let keep_paths = manifest.collect_keep_paths(&selections.selections);
    let workspaces: Vec<serde_json::Value> = keep_paths
        .iter()
        .filter(|path| {
            // Only include paths that are actual workspace packages (have a package.json)
            let full_path = project_dir.join(path).join("package.json");
            full_path.exists()
        })
        .map(|path| serde_json::Value::String(path.clone()))
        .collect();

    if !workspaces.is_empty() {
        package["workspaces"] = serde_json::Value::Array(workspaces);
    }

    // Rewrite scripts to use bun instead of pnpm
    if let Some(scripts) = package.get_mut("scripts").and_then(|s| s.as_object_mut()) {
        let script_keys: Vec<String> = scripts.keys().cloned().collect();
        for key in script_keys {
            if let Some(value) = scripts.get_mut(&key) {
                if let Some(script_str) = value.as_str() {
                    let updated = script_str
                        .replace("pnpm ", "bun ")
                        .replace("pnpx ", "bunx ")
                        .replace("npx ", "bunx ");
                    *value = serde_json::Value::String(updated);
                }
            }
        }
    }

    // Write the updated package.json
    let updated_content =
        serde_json::to_string_pretty(&package).context("Failed to serialize package.json")?;
    std::fs::write(&package_json_path, updated_content)
        .context("Failed to write package.json")?;

    tracing::info!("Rewrote package.json for bun");
    Ok(())
}

/// Generate bun workspace configuration.
/// Bun uses the "workspaces" field in package.json (already set above),
/// but we also create a bunfig.toml for any bun-specific settings.
fn generate_bun_workspace(
    project_dir: &Path,
    _manifest: &WeaveManifest,
    _selections: &UserSelections,
) -> Result<()> {
    let bunfig_content = r#"# Bun configuration for the Weave project
# See: https://bun.sh/docs/runtime/bunfig

[install]
# Use the default registry
peer = false

[install.lockfile]
# Save the lockfile
save = true
"#;

    let bunfig_path = project_dir.join("bunfig.toml");
    std::fs::write(&bunfig_path, bunfig_content)
        .context("Failed to write bunfig.toml")?;

    tracing::info!("Generated bunfig.toml");
    Ok(())
}

/// Generate a .env file with placeholders for all required environment variables
fn generate_env_file(
    project_dir: &Path,
    manifest: &WeaveManifest,
    selections: &UserSelections,
) -> Result<()> {
    let env_vars = manifest.collect_env_vars(&selections.selections);

    if env_vars.is_empty() {
        return Ok(());
    }

    let mut env_content = String::from(
        "# =============================================================================\n\
         # Environment Variables for your Weave project\n\
         # Generated by weave-cli based on your selections.\n\
         # Fill in the values below before running the project.\n\
         # =============================================================================\n\n",
    );

    for var in &env_vars {
        env_content.push_str(&format!("{}=\n", var));
    }

    let env_path = project_dir.join(".env");
    std::fs::write(&env_path, &env_content).context("Failed to write .env file")?;

    // Also write .env.example with the same content (safe to commit)
    let env_example_path = project_dir.join(".env.example");
    std::fs::write(&env_example_path, &env_content)
        .context("Failed to write .env.example file")?;

    tracing::info!("Generated .env with {} variables", env_vars.len());
    Ok(())
}

/// Filter docker-compose.yml to only include services the user selected.
/// Removes services that aren't needed based on the manifest's docker_services.
fn filter_docker_compose(
    project_dir: &Path,
    manifest: &WeaveManifest,
    selections: &UserSelections,
) -> Result<()> {
    let compose_path = project_dir.join("docker-compose.yml");

    if !compose_path.exists() {
        return Ok(());
    }

    let selected_services = manifest.collect_docker_services(&selections.selections);

    if selected_services.is_empty() {
        // No Docker services selected — remove docker-compose.yml entirely
        std::fs::remove_file(&compose_path)
            .context("Failed to remove docker-compose.yml")?;
        tracing::info!("Removed docker-compose.yml (no Docker services selected)");
        return Ok(());
    }

    let content = std::fs::read_to_string(&compose_path)
        .context("Failed to read docker-compose.yml")?;

    let mut compose: serde_json::Value = serde_yaml::from_str(&content)
        .context("Failed to parse docker-compose.yml")?;

    // Filter the services map to only include selected services
    if let Some(services) = compose.get_mut("services").and_then(|s| s.as_object_mut()) {
        let all_keys: Vec<String> = services.keys().cloned().collect();
        for key in all_keys {
            if !selected_services.contains(&key) {
                services.remove(&key);
                tracing::debug!("Removed Docker service: {}", key);
            }
        }
    }

    // Filter volumes to only keep volumes referenced by remaining services
    if let Some(services) = compose.get("services").and_then(|s| s.as_object()) {
        let used_volumes: HashSet<String> = services
            .values()
            .filter_map(|svc| svc.get("volumes"))
            .filter_map(|v| v.as_array())
            .flat_map(|arr| arr.iter())
            .filter_map(|v| v.as_str())
            .filter_map(|s| s.split(':').next())
            .filter(|s| !s.starts_with('.') && !s.starts_with('/')) // Only named volumes
            .map(|s| s.to_string())
            .collect();

        if let Some(volumes) = compose.get_mut("volumes").and_then(|v| v.as_object_mut()) {
            let all_vol_keys: Vec<String> = volumes.keys().cloned().collect();
            for key in all_vol_keys {
                if !used_volumes.contains(&key) {
                    volumes.remove(&key);
                }
            }
        }
    }

    // Write back as YAML
    let updated = serde_yaml::to_string(&compose)
        .context("Failed to serialize docker-compose.yml")?;
    std::fs::write(&compose_path, updated)
        .context("Failed to write docker-compose.yml")?;

    tracing::info!(
        "Filtered docker-compose.yml to {} services",
        selected_services.len()
    );
    Ok(())
}

/// Remove files that should not be in the scaffolded project
fn cleanup_artifacts(project_dir: &Path) -> Result<()> {
    let remove_files = [
        // pnpm artifacts (project uses bun)
        "pnpm-workspace.yaml",
        "pnpm-lock.yaml",
        ".pnpmrc",
        ".npmrc",
        // CLI-internal manifest (not useful in the scaffolded project)
        "weave.manifest.toml",
    ];

    for file in &remove_files {
        let path = project_dir.join(file);
        if path.exists() {
            std::fs::remove_file(&path)
                .with_context(|| format!("Failed to remove {}", file))?;
            tracing::debug!("Removed {}", file);
        }
    }

    tracing::info!("Cleaned up artifacts");
    Ok(())
}

/// Initialize a fresh git repository in the project directory
fn initialize_git(project_dir: &Path) -> Result<()> {
    match git2::Repository::init(project_dir) {
        Ok(_repo) => {
            tracing::info!("Initialized git repository");
        }
        Err(error) => {
            tracing::warn!("Failed to initialize git repository: {}", error);
            // Not fatal — user can init git manually
        }
    }
    Ok(())
}
