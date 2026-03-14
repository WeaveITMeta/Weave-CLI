// =============================================================================
// Downloader - Fetch template repository from GitHub or use local source
// =============================================================================
//
// Table of Contents:
// - TemplateSource: Enum for local path vs GitHub download
// - fetch_template: Main entry point — resolves source and returns local path
// - download_github_release: Download and extract a tagged release archive
// - get_latest_tag: Query GitHub API for the most recent release tag
// - cache_path: Compute the local cache directory for a given version
// =============================================================================

use crate::config::constants;
use anyhow::{Context, Result, bail};
use std::path::{Path, PathBuf};

/// Where the template source comes from
#[derive(Debug, Clone)]
pub enum TemplateSource {
    /// A local directory path (for development and testing)
    Local(PathBuf),

    /// Download from GitHub (optionally a specific version tag)
    GitHub { version: Option<String> },
}

/// Resolve the template source to a local directory path.
/// If the source is GitHub, this downloads and caches the release archive.
/// If the source is local, this validates the path exists.
pub async fn fetch_template(source: &TemplateSource) -> Result<PathBuf> {
    match source {
        TemplateSource::Local(path) => {
            if !path.exists() {
                bail!(
                    "Local template source does not exist: {}",
                    path.display()
                );
            }
            if !path.is_dir() {
                bail!(
                    "Local template source is not a directory: {}",
                    path.display()
                );
            }
            // Verify manifest exists
            let manifest_path = path.join(constants::MANIFEST_FILENAME);
            if !manifest_path.exists() {
                bail!(
                    "Template source is missing {}. Is this a valid Weave Template repository?",
                    constants::MANIFEST_FILENAME
                );
            }
            Ok(path.clone())
        }
        TemplateSource::GitHub { version } => {
            let tag = match version {
                Some(tag) => tag.clone(),
                None => get_latest_tag().await?,
            };

            // Check cache first
            let cached = cache_path(&tag);
            if cached.exists() {
                tracing::info!("Using cached template: {}", cached.display());
                return Ok(cached);
            }

            // Download from GitHub
            download_github_release(&tag, &cached).await?;
            Ok(cached)
        }
    }
}

/// Query the GitHub API for the latest release tag of the template repository
pub async fn get_latest_tag() -> Result<String> {
    let url = format!("{}/releases/latest", constants::GITHUB_API_BASE);

    let client = reqwest::Client::builder()
        .user_agent("weave-cli")
        .build()?;

    let response = client
        .get(&url)
        .send()
        .await
        .context("Failed to fetch latest release from GitHub")?;

    if !response.status().is_success() {
        // If no releases exist yet, default to downloading the main branch
        tracing::warn!("No releases found, falling back to main branch");
        return Ok("main".to_string());
    }

    let body = response
        .text()
        .await
        .context("Failed to read GitHub API response body")?;

    let json: serde_json::Value = serde_json::from_str(&body)
        .context("Failed to parse GitHub API response")?;

    let tag = json["tag_name"]
        .as_str()
        .context("Release has no tag_name")?
        .to_string();

    Ok(tag)
}

/// Download a release archive from GitHub and extract it to the cache directory
pub async fn download_github_release(tag: &str, destination: &Path) -> Result<()> {
    let url = format!(
        "{}/archive/refs/{}.tar.gz",
        constants::TEMPLATE_REPO_URL,
        if tag == "main" {
            "heads/main".to_string()
        } else {
            format!("tags/{}", tag)
        }
    );

    tracing::info!("Downloading template from: {}", url);

    let client = reqwest::Client::builder()
        .user_agent("weave-cli")
        .build()?;

    let response = client
        .get(&url)
        .send()
        .await
        .context("Failed to download template archive")?;

    if !response.status().is_success() {
        bail!(
            "Failed to download template: HTTP {}",
            response.status()
        );
    }

    let bytes = response
        .bytes()
        .await
        .context("Failed to read response body")?;

    // Create parent directory
    if let Some(parent) = destination.parent() {
        std::fs::create_dir_all(parent)
            .context("Failed to create cache directory")?;
    }

    // Create a temporary directory for extraction
    let temp_dir = tempfile::tempdir().context("Failed to create temp directory")?;

    // Decompress gzip
    let gz_decoder = flate2::read::GzDecoder::new(&bytes[..]);

    // Extract tar archive
    let mut archive = tar::Archive::new(gz_decoder);
    archive
        .unpack(temp_dir.path())
        .context("Failed to extract template archive")?;

    // GitHub archives contain a single top-level directory like "Weave-Template-v1.0.0/"
    // Find it and move its contents to the destination
    let mut entries = std::fs::read_dir(temp_dir.path())
        .context("Failed to read extracted archive")?;

    let extracted_dir = entries
        .next()
        .context("Archive is empty")?
        .context("Failed to read extracted directory")?
        .path();

    // Move the extracted directory to the final cache location
    if destination.exists() {
        std::fs::remove_dir_all(destination)
            .context("Failed to clean existing cache directory")?;
    }

    fs_extra::dir::copy(
        &extracted_dir,
        destination.parent().unwrap(),
        &fs_extra::dir::CopyOptions::new(),
    )
    .context("Failed to copy extracted template to cache")?;

    // Rename the extracted directory to the expected cache path name
    let copied_name = destination
        .parent()
        .unwrap()
        .join(extracted_dir.file_name().unwrap());
    if copied_name != *destination {
        std::fs::rename(&copied_name, destination)
            .context("Failed to rename cached template directory")?;
    }

    tracing::info!("Template cached at: {}", destination.display());

    Ok(())
}

/// Compute the local cache directory for a given template version
pub fn cache_path(version: &str) -> PathBuf {
    let cache_base = dirs::cache_dir()
        .unwrap_or_else(|| PathBuf::from(".cache"))
        .join(constants::CACHE_DIR_NAME);
    cache_base.join(version)
}

/// Determine the template source based on CLI arguments and environment variables
pub fn resolve_source(
    cli_source: Option<PathBuf>,
    cli_version: Option<String>,
) -> TemplateSource {
    // Priority 1: Explicit CLI --source flag
    if let Some(path) = cli_source {
        return TemplateSource::Local(path);
    }

    // Priority 2: WEAVE_TEMPLATE_PATH environment variable
    if let Ok(env_path) = std::env::var(constants::ENV_TEMPLATE_PATH) {
        let path = PathBuf::from(env_path);
        if path.exists() {
            return TemplateSource::Local(path);
        }
    }

    // Priority 3: GitHub download (default)
    TemplateSource::GitHub {
        version: cli_version,
    }
}
