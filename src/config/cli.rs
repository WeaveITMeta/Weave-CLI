// =============================================================================
// CLI Argument Parsing - Clap derive-based command definitions
// =============================================================================
//
// Table of Contents:
// - WeaveCommand: Top-level CLI commands (init, update, info)
// - InitArgs: Arguments for the `weave init` subcommand
// - UpdateArgs: Arguments for the `weave update` subcommand
// =============================================================================

use clap::{Parser, Subcommand};
use std::path::PathBuf;

/// Weave CLI — Full-Stack Composition Engine
///
/// Scaffold production-ready monorepo projects with an interactive terminal UI.
/// Choose your platform, backend language, auth, database, cloud, and microservices.
#[derive(Parser, Debug)]
#[command(name = "weave", version, about, long_about = None)]
pub struct WeaveCommand {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Create a new project with the interactive wizard
    Init(InitArgs),

    /// Update the CLI's cached template to the latest version
    Update(UpdateArgs),

    /// Display information about the current template cache and CLI version
    Info,
}

/// Arguments for the `weave init` subcommand
#[derive(Parser, Debug)]
pub struct InitArgs {
    /// Name of the project directory to create
    pub project_name: String,

    /// Path to a local template source directory (for development/testing)
    /// Overrides downloading from GitHub. Also settable via WEAVE_TEMPLATE_PATH.
    #[arg(long, short = 's')]
    pub source: Option<PathBuf>,

    /// Specific template version tag to use (for example, v1.0.0)
    /// Defaults to the latest release.
    #[arg(long, short = 'v')]
    pub version: Option<String>,

    /// Skip the interactive wizard and use a weave.toml config file
    #[arg(long, short = 'c')]
    pub config: Option<PathBuf>,

    /// Output directory where the project will be created
    /// Defaults to the current working directory.
    #[arg(long, short = 'o')]
    pub output: Option<PathBuf>,

    /// Skip running `bun install` after scaffolding
    #[arg(long)]
    pub skip_install: bool,

    /// Skip initializing a git repository in the new project
    #[arg(long)]
    pub skip_git: bool,
}

/// Arguments for the `weave update` subcommand
#[derive(Parser, Debug)]
pub struct UpdateArgs {
    /// Force re-download even if cache is up to date
    #[arg(long, short = 'f')]
    pub force: bool,
}
