// =============================================================================
// Constants - Static configuration values for the Weave CLI
// =============================================================================
//
// Table of Contents:
// - Template repository URLs and paths
// - Version information
// - Cache directory names
// - Default configuration values
// =============================================================================

/// GitHub organization that owns the template repository
pub const GITHUB_ORG: &str = "WeaveITMeta";

/// Template repository name on GitHub
pub const TEMPLATE_REPO: &str = "Weave-Template";

/// Full GitHub URL for the template repository
pub const TEMPLATE_REPO_URL: &str = "https://github.com/WeaveITMeta/Weave-Template";

/// GitHub API base URL for fetching release information
pub const GITHUB_API_BASE: &str = "https://api.github.com/repos/WeaveITMeta/Weave-Template";

/// Manifest filename that lives inside the template repository
pub const MANIFEST_FILENAME: &str = "weave.manifest.toml";

/// Cache directory name inside the user's system cache folder
pub const CACHE_DIR_NAME: &str = "weave";

/// Environment variable to override template source path (for local development)
pub const ENV_TEMPLATE_PATH: &str = "WEAVE_TEMPLATE_PATH";

/// Default package manager for scaffolded projects
pub const DEFAULT_PACKAGE_MANAGER: &str = "bun";

/// ASCII art logo displayed on the welcome screen
pub const LOGO: &str = r#"
 ██╗    ██╗███████╗ █████╗ ██╗   ██╗███████╗
 ██║    ██║██╔════╝██╔══██╗██║   ██║██╔════╝
 ██║ █╗ ██║█████╗  ███████║██║   ██║█████╗  
 ██║███╗██║██╔══╝  ██╔══██╗╚██╗ ██╔╝██╔══╝  
 ╚███╔███╔╝███████╗██║  ██║ ╚████╔╝ ███████╗
  ╚══╝╚══╝ ╚══════╝╚═╝  ╚═╝  ╚═══╝  ╚══════╝
"#;

/// Tagline displayed below the logo
pub const TAGLINE: &str = "Full-Stack Composition Engine";

/// Version string for display
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
