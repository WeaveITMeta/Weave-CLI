// =============================================================================
// Weave CLI — Full-Stack Composition Engine
// =============================================================================
//
// Table of Contents:
// - Module declarations
// - main(): Entry point — parse CLI args and dispatch commands
// - run_init(): Execute the `weave init` command (wizard + scaffold)
// - run_update(): Execute the `weave update` command (refresh cache)
// - run_info(): Execute the `weave info` command (display cache/version info)
// - run_wizard(): Launch the Ratatui interactive wizard
// - run_scaffold(): Execute template download, prune, and config generation
// =============================================================================

mod config;
mod core;
mod engine;
mod ui;

use anyhow::{Context, Result};
use clap::Parser;
use config::cli::{Commands, InitArgs, UpdateArgs, WeaveCommand};
use config::constants;
use core::manifest;
use engine::{downloader, generator, pruner};
use std::path::PathBuf;
use ui::app::{App, AppScreen};

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing for structured logging
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive("weave_init=info".parse().unwrap()),
        )
        .with_target(false)
        .init();

    let cli = WeaveCommand::parse();

    match cli.command {
        Commands::Init(args) => run_init(args).await,
        Commands::Update(args) => run_update(args).await,
        Commands::Info => run_info(),
    }
}

/// Execute the `weave init` command.
/// This launches the interactive wizard, then scaffolds the project.
async fn run_init(args: InitArgs) -> Result<()> {
    let project_name = args.project_name.clone();

    // Determine where to create the project
    let output_dir = args
        .output
        .clone()
        .unwrap_or_else(|| std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")));
    let project_dir = output_dir.join(&project_name);

    // Check if project directory already exists
    if project_dir.exists() {
        anyhow::bail!(
            "Directory '{}' already exists. Choose a different name or remove it first.",
            project_dir.display()
        );
    }

    // Resolve template source (local path or GitHub download)
    let source = downloader::resolve_source(args.source.clone(), args.version.clone());

    // Fetch the template (download if needed, or validate local path)
    println!("Resolving template source...");
    let template_path = downloader::fetch_template(&source).await?;

    // Parse the manifest from the template
    let manifest_path = template_path.join(constants::MANIFEST_FILENAME);
    let manifest = manifest::parse_manifest_file(&manifest_path)
        .context("Failed to parse template manifest. Is this a valid Weave Template?")?;

    // Check if user provided a config file to skip the wizard
    let selections = if let Some(config_path) = &args.config {
        let config_content = std::fs::read_to_string(config_path)
            .with_context(|| format!("Failed to read config file: {}", config_path.display()))?;
        core::selections::UserSelections::from_toml_string(&config_content)
            .context("Failed to parse config file")?
    } else {
        // Launch the interactive Ratatui wizard
        run_wizard(manifest.clone(), project_name.clone())?
    };

    // Run the scaffolding process
    println!();
    run_scaffold(
        &template_path,
        &project_dir,
        &manifest,
        &selections,
        args.skip_install,
        args.skip_git,
    )?;

    Ok(())
}

/// Launch the Ratatui interactive terminal wizard.
/// Returns the user's selections when they confirm on the summary screen.
fn run_wizard(
    manifest: core::manifest::WeaveManifest,
    project_name: String,
) -> Result<core::selections::UserSelections> {
    // Initialize the terminal
    let mut terminal = ratatui::init();

    // Create the app state
    let mut app = App::new(manifest, project_name);

    // Run the event loop until the user reaches the scaffold step or quits
    loop {
        terminal.draw(|frame| {
            let area = frame.area();
            match &app.screen {
                AppScreen::Welcome => {
                    ui::screens::render_welcome_screen(frame, area);
                }
                AppScreen::Selection(index) => {
                    let index = *index;
                    let category = &app.categories[index].clone();
                    let step_current = index + 1;
                    let step_total = app.categories.len();
                    ui::screens::render_selection_screen(
                        frame,
                        area,
                        category,
                        &mut app.list_states[index],
                        step_current,
                        step_total,
                    );
                }
                AppScreen::Summary => {
                    ui::screens::render_summary_screen(frame, area, &app.selections);
                }
                AppScreen::Progress | AppScreen::Complete => {
                    // These screens are handled outside the wizard loop
                }
            }
        })?;

        // Handle input
        if let crossterm::event::Event::Key(key) = crossterm::event::read()? {
            if key.kind != crossterm::event::KeyEventKind::Press {
                continue;
            }

            match &app.screen {
                AppScreen::Welcome => match key.code {
                    crossterm::event::KeyCode::Enter => {
                        app.screen = AppScreen::Selection(0);
                    }
                    crossterm::event::KeyCode::Char('q') => {
                        ratatui::restore();
                        std::process::exit(0);
                    }
                    _ => {}
                },

                AppScreen::Selection(index) => {
                    let index = *index;
                    match key.code {
                        crossterm::event::KeyCode::Up => {
                            app.list_states[index].previous();
                        }
                        crossterm::event::KeyCode::Down => {
                            app.list_states[index].next();
                        }
                        crossterm::event::KeyCode::Char(' ') => {
                            app.list_states[index].toggle();
                        }
                        crossterm::event::KeyCode::Char('a') => {
                            if app.list_states[index].mode
                                == core::selections::SelectionMode::Multi
                            {
                                let all_checked =
                                    app.list_states[index].checked.iter().all(|c| *c);
                                for checked in &mut app.list_states[index].checked {
                                    *checked = !all_checked;
                                }
                            }
                        }
                        crossterm::event::KeyCode::Enter => {
                            let category = &app.categories[index].clone();
                            let mode =
                                core::selections::UserSelections::selection_mode_for(category);

                            // For single-select, Enter auto-selects the highlighted item
                            if mode == core::selections::SelectionMode::Single {
                                app.list_states[index].toggle();
                            }

                            let selected = app.list_states[index].selected_keys();

                            // Require at least one selection for single-select and backends
                            let requires_selection = mode == core::selections::SelectionMode::Single
                                || category == "backends";
                            if requires_selection && selected.is_empty() {
                                continue;
                            }

                            app.selections.set_multi(category, selected);

                            if index + 1 < app.categories.len() {
                                app.screen = AppScreen::Selection(index + 1);
                            } else {
                                app.screen = AppScreen::Summary;
                            }
                        }
                        crossterm::event::KeyCode::Esc => {
                            if index == 0 {
                                app.screen = AppScreen::Welcome;
                            } else {
                                app.screen = AppScreen::Selection(index - 1);
                            }
                        }
                        crossterm::event::KeyCode::Char('q') => {
                            ratatui::restore();
                            std::process::exit(0);
                        }
                        _ => {}
                    }
                }

                AppScreen::Summary => match key.code {
                    crossterm::event::KeyCode::Enter => {
                        // User confirmed — exit wizard and return selections
                        break;
                    }
                    crossterm::event::KeyCode::Esc => {
                        let last_index = app.categories.len() - 1;
                        app.screen = AppScreen::Selection(last_index);
                    }
                    crossterm::event::KeyCode::Char('q') => {
                        ratatui::restore();
                        std::process::exit(0);
                    }
                    _ => {}
                },

                _ => {}
            }
        }
    }

    // Restore the terminal before proceeding to scaffolding output
    ratatui::restore();

    Ok(app.selections)
}

/// Execute the scaffolding process: copy template, prune, generate configs.
fn run_scaffold(
    template_path: &std::path::Path,
    project_dir: &std::path::Path,
    manifest: &core::manifest::WeaveManifest,
    selections: &core::selections::UserSelections,
    skip_install: bool,
    skip_git: bool,
) -> Result<()> {
    // Step 1: Create project directory
    println!("Creating project directory...");
    std::fs::create_dir_all(project_dir)
        .context("Failed to create project directory")?;

    // Step 2: Collect keep paths from selections
    let keep_paths = manifest.collect_keep_paths(&selections.selections);
    println!(
        "Keeping {} directory paths based on your selections.",
        keep_paths.len()
    );

    // Step 3: Copy and prune template
    println!("Copying and pruning template...");
    pruner::prune_template(template_path, project_dir, &keep_paths)?;

    // Step 4: Generate configuration files (package.json, .env, bunfig.toml, etc.)
    println!("Generating configuration files...");
    generator::post_scaffold(project_dir, manifest, selections, skip_git)?;

    // Step 5: Run bun install (unless skipped)
    if !skip_install {
        println!("Running bun install...");
        let install_status = std::process::Command::new("bun")
            .arg("install")
            .current_dir(project_dir)
            .status();

        match install_status {
            Ok(status) if status.success() => {
                println!("Dependencies installed successfully.");
            }
            Ok(status) => {
                eprintln!(
                    "Warning: bun install exited with status {}. You may need to run it manually.",
                    status
                );
            }
            Err(error) => {
                eprintln!(
                    "Warning: Could not run bun install ({}). Make sure bun is installed, then run: cd {} && bun install",
                    error,
                    project_dir.display()
                );
            }
        }
    }

    // Success output
    println!();
    println!("  Project scaffolded successfully!");
    println!("  {}", project_dir.display());
    println!();
    println!("  Next steps:");
    println!("    1. cd {}", selections.project_name);
    println!("    2. Copy .env.example to .env and fill in your keys");
    if skip_install {
        println!("    3. bun install");
        println!("    4. bun dev");
    } else {
        println!("    3. bun dev");
    }
    println!();
    println!(
        "  Your selections are saved in weave.toml for reproducibility."
    );

    Ok(())
}

/// Execute the `weave update` command — refresh the cached template
async fn run_update(args: UpdateArgs) -> Result<()> {
    println!("Checking for template updates...");

    let latest_tag = downloader::get_latest_tag().await?;
    let cached = downloader::cache_path(&latest_tag);

    if cached.exists() && !args.force {
        println!("Cache is up to date ({})", latest_tag);
        return Ok(());
    }

    if cached.exists() && args.force {
        println!("Force re-downloading {}...", latest_tag);
        std::fs::remove_dir_all(&cached).context("Failed to remove cached template")?;
    }

    let source = downloader::TemplateSource::GitHub {
        version: Some(latest_tag.clone()),
    };
    downloader::fetch_template(&source).await?;
    println!("Template updated to {}", latest_tag);

    Ok(())
}

/// Execute the `weave info` command — display version and cache info
fn run_info() -> Result<()> {
    println!("Weave CLI v{}", constants::VERSION);
    println!();
    println!("Template Repository: {}", constants::TEMPLATE_REPO_URL);
    println!("Cache Directory:     {}", cache_dir_display());
    println!();

    // List cached versions
    let cache_base = dirs::cache_dir()
        .unwrap_or_else(|| PathBuf::from(".cache"))
        .join(constants::CACHE_DIR_NAME);

    if cache_base.exists() {
        println!("Cached versions:");
        let mut entries: Vec<_> = std::fs::read_dir(&cache_base)?
            .filter_map(|e| e.ok())
            .filter(|e| e.path().is_dir())
            .collect();
        entries.sort_by_key(|e| e.file_name());

        if entries.is_empty() {
            println!("  (none)");
        } else {
            for entry in entries {
                println!("  - {}", entry.file_name().to_string_lossy());
            }
        }
    } else {
        println!("No cached templates found.");
    }

    Ok(())
}

/// Get the cache directory path as a display string
fn cache_dir_display() -> String {
    dirs::cache_dir()
        .unwrap_or_else(|| PathBuf::from(".cache"))
        .join(constants::CACHE_DIR_NAME)
        .display()
        .to_string()
}
