// =============================================================================
// Pruner - Remove unneeded directories and files based on user selections
// =============================================================================
//
// Table of Contents:
// - prune_template: Main entry point — copy template and remove unneeded parts
// - resolve_keep_paths: Expand glob patterns into concrete directory paths
// - always_keep: Directories and files that are always preserved
// - should_prune: Check if a given path should be removed
// =============================================================================

use anyhow::{Context, Result};
use std::collections::HashSet;
use std::path::{Path, PathBuf};

/// Directories and files that are ALWAYS kept regardless of selections.
/// These are root-level configuration and documentation files.
const ALWAYS_KEEP: &[&str] = &[
    ".gitignore",
    ".env.example",
    "package.json",
    "tsconfig.json",
    "README.md",
    "LICENSE",
    "weave.manifest.toml",
    "weave.toml",
];

/// Top-level directories that are prunable (contain selectable content).
/// If a directory is not in this list, it is always kept.
const PRUNABLE_ROOTS: &[&str] = &[
    "apps",
    "packages",
    "microservices",
    "terraform",
    "database",
    "monitoring",
    "supabase",
];

/// Copy the template source to the project destination, then prune
/// directories that were not selected by the user.
pub fn prune_template(
    source: &Path,
    destination: &Path,
    keep_paths: &[String],
) -> Result<()> {
    // Step 1: Resolve glob patterns in keep_paths to concrete paths
    let resolved_keeps = resolve_keep_paths(source, keep_paths)?;

    // Step 2: Copy the entire template to the destination
    tracing::info!("Copying template to {}", destination.display());

    let copy_options = fs_extra::dir::CopyOptions {
        overwrite: true,
        skip_exist: false,
        copy_inside: true,
        content_only: true,
        ..Default::default()
    };

    fs_extra::dir::copy(source, destination, &copy_options)
        .context("Failed to copy template to project directory")?;

    // Step 3: Walk the destination and remove directories that should be pruned
    tracing::info!("Pruning unselected directories...");
    prune_directory(destination, destination, &resolved_keeps)?;

    // Step 4: Clean up empty directories left after pruning
    remove_empty_dirs(destination)?;

    Ok(())
}

/// Recursively prune directories that are not in the keep set
fn prune_directory(
    root: &Path,
    current: &Path,
    keeps: &HashSet<PathBuf>,
) -> Result<()> {
    if !current.is_dir() {
        return Ok(());
    }

    let entries: Vec<_> = std::fs::read_dir(current)
        .context("Failed to read directory during pruning")?
        .filter_map(|e| e.ok())
        .collect();

    for entry in entries {
        let path = entry.path();
        let relative = path
            .strip_prefix(root)
            .unwrap_or(&path)
            .to_string_lossy()
            .replace('\\', "/");

        // Skip always-keep files
        if ALWAYS_KEEP.iter().any(|keep| relative == *keep) {
            continue;
        }

        // Check if this is inside a prunable root
        let is_prunable = PRUNABLE_ROOTS
            .iter()
            .any(|prunable_root| relative.starts_with(prunable_root));

        if !is_prunable {
            continue;
        }

        if path.is_dir() {
            // Check if this directory or any of its descendants are in the keep set
            let should_keep = keeps.iter().any(|keep| {
                let keep_str = keep.to_string_lossy().replace('\\', "/");
                // Keep if the directory IS a kept path, or is an ancestor/descendant of one
                keep_str.starts_with(relative.as_str()) || relative.starts_with(keep_str.as_str())
            });

            if should_keep {
                // Recurse deeper — some subdirectories may still need pruning
                prune_directory(root, &path, keeps)?;
            } else {
                // This entire directory tree should be removed
                tracing::debug!("Pruning: {}", relative);
                std::fs::remove_dir_all(&path)
                    .with_context(|| format!("Failed to remove directory: {}", path.display()))?;
            }
        }
    }

    Ok(())
}

/// Resolve glob patterns in keep paths to concrete relative paths
fn resolve_keep_paths(source: &Path, keep_paths: &[String]) -> Result<HashSet<PathBuf>> {
    let mut resolved = HashSet::new();

    for pattern in keep_paths {
        // Replace forward slashes for cross-platform compatibility
        let normalized = pattern.replace('\\', "/");

        if normalized.contains('*') {
            // Expand glob pattern relative to the source directory
            let full_pattern = source.join(&normalized).to_string_lossy().to_string();
            for entry in glob::glob(&full_pattern).context("Invalid glob pattern")? {
                if let Ok(path) = entry {
                    if let Ok(relative) = path.strip_prefix(source) {
                        resolved.insert(relative.to_path_buf());
                    }
                }
            }
        } else {
            // Direct path — add as-is
            resolved.insert(PathBuf::from(&normalized));
        }
    }

    Ok(resolved)
}

/// Recursively remove empty directories
fn remove_empty_dirs(path: &Path) -> Result<bool> {
    if !path.is_dir() {
        return Ok(false);
    }

    let entries: Vec<_> = std::fs::read_dir(path)?
        .filter_map(|e| e.ok())
        .collect();

    for entry in &entries {
        let child_path = entry.path();
        if child_path.is_dir() {
            remove_empty_dirs(&child_path)?;
        }
    }

    // Re-check if directory is now empty
    let remaining: Vec<_> = std::fs::read_dir(path)?
        .filter_map(|e| e.ok())
        .collect();

    if remaining.is_empty() {
        std::fs::remove_dir(path)?;
        return Ok(true);
    }

    Ok(false)
}
