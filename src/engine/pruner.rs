// =============================================================================
// Pruner - Copy only selected content from template to project directory
// =============================================================================
//
// Table of Contents:
// - prune_template: Main entry point — selective copy based on user choices
// - selective_copy: Recursive copy that only copies kept paths (never copies junk)
// - resolve_keep_paths: Expand glob patterns into concrete directory paths
// - is_path_selected: Check whether a relative path should be included
// =============================================================================

use anyhow::{Context, Result};
use std::collections::HashSet;
use std::path::{Path, PathBuf};

/// Directories and files that are ALWAYS kept regardless of selections.
/// These are root-level configuration and documentation files.
const ALWAYS_KEEP: &[&str] = &[
    ".gitignore",
    ".env.example",
    ".env",
    "package.json",
    "tsconfig.json",
    "README.md",
    "LICENSE",
    "docker-compose.yml",
    "weave.manifest.toml",
    "weave.toml",
    "bun.lockb",
    "bunfig.toml",
    "pnpm-workspace.yaml",
];

/// Top-level directories that are prunable (contain selectable content).
/// Only directories inside these roots are subject to keep-path filtering.
/// Everything outside these roots is always copied (e.g., .github, docs, scripts).
const PRUNABLE_ROOTS: &[&str] = &[
    "apps",
    "packages",
    "microservices",
    "terraform",
    "database",
    "monitoring",
    "supabase",
];

/// Directories that are NEVER copied from the template source.
/// Build artifacts, dependency caches, and VCS metadata.
const SKIP_DIRECTORIES: &[&str] = &[
    "node_modules",
    ".git",
    ".turbo",
    ".next",
    ".nuxt",
    ".expo",
    "dist",
    "build",
    ".cache",
    ".pnpm-store",
    "__pycache__",
    "target",
    ".VSCodeCounter",
];

/// Selectively copy the template to the destination.
/// Only copies files that match the user's selections — never copies
/// anything that would be immediately pruned. This is a single-pass
/// operation: no copy-then-delete cycle needed.
pub fn prune_template(
    source: &Path,
    destination: &Path,
    keep_paths: &[String],
) -> Result<()> {
    // Resolve glob patterns in keep_paths to concrete relative paths
    let resolved_keeps = resolve_keep_paths(source, keep_paths)?;

    tracing::info!(
        "Selective copy: {} keep paths resolved",
        resolved_keeps.len()
    );
    for keep in &resolved_keeps {
        tracing::debug!("  keep: {}", keep.display());
    }

    // Single-pass selective copy
    selective_copy(source, destination, source, &resolved_keeps)
        .context("Failed to copy template to project directory")?;

    Ok(())
}

/// Recursively copy a directory tree, applying three filters:
/// 1. SKIP_DIRECTORIES — never copy node_modules, .git, etc.
/// 2. PRUNABLE_ROOTS — directories inside these roots must match a keep path
/// 3. Everything outside prunable roots is always copied
fn selective_copy(
    source_root: &Path,
    destination_root: &Path,
    current_source: &Path,
    keeps: &HashSet<PathBuf>,
) -> Result<()> {
    let destination_dir = if current_source == source_root {
        destination_root.to_path_buf()
    } else {
        let relative = current_source.strip_prefix(source_root).unwrap();
        destination_root.join(relative)
    };

    if !destination_dir.exists() {
        std::fs::create_dir_all(&destination_dir)
            .with_context(|| format!("Failed to create directory: {}", destination_dir.display()))?;
    }

    let entries = std::fs::read_dir(current_source)
        .with_context(|| format!("Failed to read directory: {}", current_source.display()))?;

    for entry in entries {
        let entry = entry.context("Failed to read directory entry")?;
        let file_name = entry.file_name();
        let file_name_str = file_name.to_string_lossy();
        let source_path = entry.path();

        // Compute the relative path from the template root (forward-slash normalized)
        let relative = source_path
            .strip_prefix(source_root)
            .unwrap_or(&source_path);
        let relative_str = relative.to_string_lossy().replace('\\', "/");

        if source_path.is_dir() {
            // Filter 1: Skip build artifacts and dependency caches
            if SKIP_DIRECTORIES.iter().any(|skip| file_name_str == *skip) {
                tracing::debug!("Skipping excluded: {}", relative_str);
                continue;
            }

            // Filter 2: Check if this directory is inside a prunable root
            let in_prunable_root = PRUNABLE_ROOTS
                .iter()
                .any(|root| relative_str.starts_with(root));

            if in_prunable_root {
                // Only copy if this directory is selected (or is an ancestor/descendant of a selected path)
                if is_path_selected(&relative_str, keeps) {
                    selective_copy(source_root, destination_root, &source_path, keeps)?;
                } else {
                    tracing::debug!("Skipping unselected: {}", relative_str);
                }
            } else {
                // Outside prunable roots — always copy
                selective_copy(source_root, destination_root, &source_path, keeps)?;
            }
        } else {
            // Files: copy if parent directory passed the filter (we're already inside it)
            // Root-level files: always copy if in ALWAYS_KEEP, or if not inside a prunable root
            if current_source == source_root {
                // Root-level file — only copy if it's in ALWAYS_KEEP or not filterable
                let is_kept = ALWAYS_KEEP.iter().any(|keep| file_name_str == *keep);
                let is_prunable_file = PRUNABLE_ROOTS
                    .iter()
                    .any(|root| file_name_str == *root);

                if !is_kept && is_prunable_file {
                    continue;
                }
            }

            let dest_file = destination_dir.join(&file_name);
            std::fs::copy(&source_path, &dest_file).with_context(|| {
                format!(
                    "Failed to copy: {} -> {}",
                    source_path.display(),
                    dest_file.display()
                )
            })?;
        }
    }

    Ok(())
}

/// Check if a relative path matches any keep path.
/// Returns true if the path IS a keep path, is an ancestor of one, or is a descendant of one.
fn is_path_selected(relative: &str, keeps: &HashSet<PathBuf>) -> bool {
    keeps.iter().any(|keep| {
        let keep_str = keep.to_string_lossy().replace('\\', "/");
        // Path is a kept directory, or is an ancestor of a kept path, or is inside a kept path
        keep_str.starts_with(relative) || relative.starts_with(keep_str.as_str())
    })
}

/// Resolve glob patterns in keep paths to concrete relative paths
fn resolve_keep_paths(source: &Path, keep_paths: &[String]) -> Result<HashSet<PathBuf>> {
    let mut resolved = HashSet::new();

    for pattern in keep_paths {
        // Normalize forward slashes for cross-platform compatibility
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
