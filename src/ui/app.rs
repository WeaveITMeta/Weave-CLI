// =============================================================================
// App - Application state machine and screen management for the wizard
// =============================================================================
//
// Table of Contents:
// - AppState: Current screen/phase of the wizard
// - App: Root application struct holding all state
// - Screen navigation (next, previous, handle input)
// - Main render dispatch (delegates to screen renderers)
// - Main event loop (crossterm events → state transitions)
// =============================================================================

use super::{screens, widgets::SelectionListState};
use crate::core::manifest::WeaveManifest;
use crate::core::selections::{SelectionMode, UserSelections};
use anyhow::Result;
use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use ratatui::{DefaultTerminal, Frame};

/// Which screen the wizard is currently showing
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AppScreen {
    /// Welcome screen with logo
    Welcome,

    /// Category selection screen (index into the category order)
    Selection(usize),

    /// Summary screen showing all choices before scaffolding
    Summary,

    /// Progress screen during scaffolding
    Progress,

    /// Completion screen with next steps
    Complete,
}

/// Root application state
pub struct App {
    /// Whether the application should exit
    pub should_quit: bool,

    /// Current screen being displayed
    pub screen: AppScreen,

    /// The parsed manifest from the template repository
    pub manifest: WeaveManifest,

    /// User's selections throughout the wizard
    pub selections: UserSelections,

    /// Selection list states for each category screen (one per category)
    pub list_states: Vec<SelectionListState>,

    /// Category keys in wizard display order
    pub categories: Vec<String>,

    /// Progress percentage (0-100) during scaffolding
    pub progress_percent: u16,

    /// Current status message during scaffolding
    pub progress_message: String,

    /// Log lines accumulated during scaffolding
    pub progress_log: Vec<String>,

    /// Path where the project was scaffolded (set after completion)
    pub project_path: String,
}

impl App {
    /// Create a new App from a parsed manifest and project name
    pub fn new(manifest: WeaveManifest, project_name: String) -> Self {
        let categories: Vec<String> = WeaveManifest::category_order()
            .iter()
            .map(|s| s.to_string())
            .collect();

        // Build a SelectionListState for each category from the manifest
        let list_states: Vec<SelectionListState> = categories
            .iter()
            .map(|category| {
                let entries = manifest.get_category_entries(category);
                let keys: Vec<String> = entries.iter().map(|(k, _)| (*k).clone()).collect();
                let labels: Vec<String> = entries.iter().map(|(_, e)| e.label.clone()).collect();
                let descriptions: Vec<String> = entries
                    .iter()
                    .map(|(_, e)| {
                        e.description
                            .clone()
                            .unwrap_or_else(|| "No description available.".to_string())
                    })
                    .collect();
                let mode = UserSelections::selection_mode_for(category);
                SelectionListState::new(keys, labels, descriptions, mode)
            })
            .collect();

        Self {
            should_quit: false,
            screen: AppScreen::Welcome,
            manifest,
            selections: UserSelections::new(project_name),
            list_states,
            categories,
            progress_percent: 0,
            progress_message: String::new(),
            progress_log: Vec::new(),
            project_path: String::new(),
        }
    }

    /// Run the main application event loop
    pub fn run(&mut self, terminal: &mut DefaultTerminal) -> Result<()> {
        while !self.should_quit {
            terminal.draw(|frame| self.render(frame))?;
            self.handle_events()?;
        }
        Ok(())
    }

    /// Dispatch rendering to the appropriate screen
    fn render(&mut self, frame: &mut Frame) {
        let area = frame.area();

        match &self.screen {
            AppScreen::Welcome => {
                screens::render_welcome_screen(frame, area);
            }
            AppScreen::Selection(index) => {
                let index = *index;
                let category = &self.categories[index].clone();
                let step_current = index + 1;
                let step_total = self.categories.len();
                screens::render_selection_screen(
                    frame,
                    area,
                    category,
                    &mut self.list_states[index],
                    step_current,
                    step_total,
                );
            }
            AppScreen::Summary => {
                screens::render_summary_screen(frame, area, &self.selections);
            }
            AppScreen::Progress => {
                screens::render_progress_screen(
                    frame,
                    area,
                    &self.progress_message,
                    self.progress_percent,
                    &self.progress_log,
                );
            }
            AppScreen::Complete => {
                screens::render_complete_screen(
                    frame,
                    area,
                    &self.selections.project_name,
                    &self.project_path,
                );
            }
        }
    }

    /// Handle crossterm input events and transition state
    fn handle_events(&mut self) -> Result<()> {
        if let Event::Key(key) = event::read()? {
            // Only handle key press events (not release or repeat)
            if key.kind != KeyEventKind::Press {
                return Ok(());
            }

            match &self.screen {
                AppScreen::Welcome => match key.code {
                    KeyCode::Enter => {
                        self.screen = AppScreen::Selection(0);
                    }
                    KeyCode::Char('q') => {
                        self.should_quit = true;
                    }
                    _ => {}
                },

                AppScreen::Selection(index) => {
                    let index = *index;
                    match key.code {
                        KeyCode::Up => {
                            self.list_states[index].previous();
                        }
                        KeyCode::Down => {
                            self.list_states[index].next();
                        }
                        KeyCode::Char(' ') => {
                            self.list_states[index].toggle();
                        }
                        KeyCode::Char('a') => {
                            // Select all (only for multi-select)
                            if self.list_states[index].mode == SelectionMode::Multi {
                                let all_checked =
                                    self.list_states[index].checked.iter().all(|c| *c);
                                for checked in &mut self.list_states[index].checked {
                                    *checked = !all_checked;
                                }
                            }
                        }
                        KeyCode::Enter => {
                            // Save selections for this category
                            let category = &self.categories[index];
                            let selected = self.list_states[index].selected_keys();

                            // For required categories, ensure at least one selection
                            let mode = UserSelections::selection_mode_for(category);
                            if mode == SelectionMode::Single && selected.is_empty() {
                                // Don't advance — user must select one
                                return Ok(());
                            }

                            self.selections.set_multi(category, selected);

                            // Advance to next category or summary
                            if index + 1 < self.categories.len() {
                                self.screen = AppScreen::Selection(index + 1);
                            } else {
                                self.screen = AppScreen::Summary;
                            }
                        }
                        KeyCode::Esc => {
                            // Go back to previous screen
                            if index == 0 {
                                self.screen = AppScreen::Welcome;
                            } else {
                                self.screen = AppScreen::Selection(index - 1);
                            }
                        }
                        KeyCode::Char('q') => {
                            self.should_quit = true;
                        }
                        _ => {}
                    }
                }

                AppScreen::Summary => match key.code {
                    KeyCode::Enter => {
                        // Transition to progress screen — scaffolding will be triggered by main
                        self.screen = AppScreen::Progress;
                    }
                    KeyCode::Esc => {
                        // Go back to last selection category
                        let last_index = self.categories.len() - 1;
                        self.screen = AppScreen::Selection(last_index);
                    }
                    KeyCode::Char('q') => {
                        self.should_quit = true;
                    }
                    _ => {}
                },

                AppScreen::Progress => {
                    // No input during progress — non-interactive
                }

                AppScreen::Complete => match key.code {
                    KeyCode::Enter | KeyCode::Char('q') => {
                        self.should_quit = true;
                    }
                    _ => {}
                },
            }
        }

        Ok(())
    }

    /// Check if the app is on the progress screen and ready to scaffold
    pub fn is_ready_to_scaffold(&self) -> bool {
        self.screen == AppScreen::Progress && self.progress_percent == 0
    }

    /// Update progress during scaffolding (called from the engine)
    pub fn update_progress(&mut self, percent: u16, message: &str) {
        self.progress_percent = percent;
        self.progress_message = message.to_string();
    }

    /// Add a log line during scaffolding
    pub fn add_log(&mut self, line: String) {
        self.progress_log.push(line);
    }

    /// Transition to the complete screen
    pub fn complete(&mut self, project_path: String) {
        self.project_path = project_path;
        self.screen = AppScreen::Complete;
    }
}
