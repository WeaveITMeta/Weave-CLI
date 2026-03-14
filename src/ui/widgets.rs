// =============================================================================
// Widgets - Reusable UI components for the Ratatui terminal interface
// =============================================================================
//
// Table of Contents:
// - SelectionList: Radio/checkbox list widget with keyboard navigation
// - PreviewPanel: Right-side panel showing details of the highlighted item
// - KeyHints: Bottom bar showing available keyboard shortcuts
// - render_selection_list: Draw a selection list into a Rect
// - render_preview_panel: Draw a preview panel into a Rect
// - render_key_hints: Draw keyboard shortcut hints into a Rect
// =============================================================================

use super::theme;
use crate::core::selections::SelectionMode;
use ratatui::{
    layout::Rect,
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph, Wrap},
    Frame,
};

/// State for a selection list widget (tracks cursor position and checked items)
#[derive(Debug, Clone)]
pub struct SelectionListState {
    /// Currently highlighted index
    pub cursor: usize,

    /// Ratatui list state for scroll tracking
    pub list_state: ListState,

    /// Set of indices that are checked (for multi-select)
    pub checked: Vec<bool>,

    /// Whether this is single-select, multi-select, or optional single
    pub mode: SelectionMode,

    /// Option labels for display
    pub labels: Vec<String>,

    /// Option keys for selection tracking
    pub keys: Vec<String>,

    /// Option descriptions for the preview panel
    pub descriptions: Vec<String>,
}

impl SelectionListState {
    /// Create a new selection list from manifest entries
    pub fn new(
        keys: Vec<String>,
        labels: Vec<String>,
        descriptions: Vec<String>,
        mode: SelectionMode,
    ) -> Self {
        let count = keys.len();
        let mut list_state = ListState::default();
        list_state.select(Some(0));
        Self {
            cursor: 0,
            list_state,
            checked: vec![false; count],
            mode,
            labels,
            keys,
            descriptions,
        }
    }

    /// Move cursor up
    pub fn previous(&mut self) {
        if self.keys.is_empty() {
            return;
        }
        if self.cursor == 0 {
            self.cursor = self.keys.len() - 1;
        } else {
            self.cursor -= 1;
        }
        self.list_state.select(Some(self.cursor));
    }

    /// Move cursor down
    pub fn next(&mut self) {
        if self.keys.is_empty() {
            return;
        }
        self.cursor = (self.cursor + 1) % self.keys.len();
        self.list_state.select(Some(self.cursor));
    }

    /// Toggle the current item (for multi-select) or select it (for single-select)
    pub fn toggle(&mut self) {
        if self.keys.is_empty() {
            return;
        }
        match self.mode {
            SelectionMode::Single => {
                // Uncheck all, then check the current one
                for checked in &mut self.checked {
                    *checked = false;
                }
                self.checked[self.cursor] = true;
            }
            SelectionMode::OptionalSingle => {
                // Toggle current; if enabling, uncheck all others first
                let currently_checked = self.checked[self.cursor];
                for checked in &mut self.checked {
                    *checked = false;
                }
                self.checked[self.cursor] = !currently_checked;
            }
            SelectionMode::Multi => {
                self.checked[self.cursor] = !self.checked[self.cursor];
            }
        }
    }

    /// Get the keys of all selected (checked) items
    pub fn selected_keys(&self) -> Vec<String> {
        self.keys
            .iter()
            .zip(self.checked.iter())
            .filter(|(_, checked)| **checked)
            .map(|(key, _)| key.clone())
            .collect()
    }

    /// Check if at least one item is selected (for required categories)
    pub fn has_selection(&self) -> bool {
        self.checked.iter().any(|c| *c)
    }

    /// Get the description of the currently highlighted item
    pub fn current_description(&self) -> &str {
        if self.keys.is_empty() {
            return "";
        }
        &self.descriptions[self.cursor]
    }

    /// Get the label of the currently highlighted item
    pub fn current_label(&self) -> &str {
        if self.keys.is_empty() {
            return "";
        }
        &self.labels[self.cursor]
    }
}

/// Render a selection list into the given area
pub fn render_selection_list(
    frame: &mut Frame,
    area: Rect,
    state: &mut SelectionListState,
    title: &str,
    is_focused: bool,
) {
    let border_style = if is_focused {
        theme::active_border_style()
    } else {
        theme::inactive_border_style()
    };

    let items: Vec<ListItem> = state
        .labels
        .iter()
        .enumerate()
        .map(|(index, label)| {
            let checkbox = match state.mode {
                SelectionMode::Single | SelectionMode::OptionalSingle => {
                    if state.checked[index] {
                        "◉ "
                    } else {
                        "○ "
                    }
                }
                SelectionMode::Multi => {
                    if state.checked[index] {
                        "☑ "
                    } else {
                        "☐ "
                    }
                }
            };

            let style = if state.checked[index] {
                theme::checked_style()
            } else {
                theme::body_style()
            };

            ListItem::new(Line::from(vec![
                Span::styled(checkbox, style),
                Span::styled(label.clone(), style),
            ]))
        })
        .collect();

    let list = List::new(items)
        .block(
            Block::default()
                .title(format!(" {} ", title))
                .title_style(theme::heading_style())
                .borders(Borders::ALL)
                .border_style(border_style),
        )
        .highlight_style(theme::selected_style())
        .highlight_symbol("▶ ");

    frame.render_stateful_widget(list, area, &mut state.list_state);
}

/// Render a preview panel showing details of the highlighted item
pub fn render_preview_panel(
    frame: &mut Frame,
    area: Rect,
    title: &str,
    description: &str,
    is_focused: bool,
) {
    let border_style = if is_focused {
        theme::active_border_style()
    } else {
        theme::inactive_border_style()
    };

    let text = if description.is_empty() {
        "No description available.".to_string()
    } else {
        description.to_string()
    };

    let paragraph = Paragraph::new(text)
        .style(theme::body_style())
        .block(
            Block::default()
                .title(format!(" {} ", title))
                .title_style(theme::heading_style())
                .borders(Borders::ALL)
                .border_style(border_style),
        )
        .wrap(Wrap { trim: true });

    frame.render_widget(paragraph, area);
}

/// Render keyboard shortcut hints at the bottom of the screen
pub fn render_key_hints(frame: &mut Frame, area: Rect, hints: &[(&str, &str)]) {
    let spans: Vec<Span> = hints
        .iter()
        .enumerate()
        .flat_map(|(index, (key, description))| {
            let mut result = vec![
                Span::styled(
                    format!(" {} ", key),
                    Style::default()
                        .fg(theme::HEADING)
                        .bg(theme::HIGHLIGHT_BACKGROUND)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled(format!(" {} ", description), theme::muted_style()),
            ];
            if index < hints.len() - 1 {
                result.push(Span::styled(" │ ", theme::muted_style()));
            }
            result
        })
        .collect();

    let paragraph = Paragraph::new(Line::from(spans));
    frame.render_widget(paragraph, area);
}
