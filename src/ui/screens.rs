// =============================================================================
// Screens - Individual wizard screens for the Ratatui terminal interface
// =============================================================================
//
// Table of Contents:
// - render_welcome_screen: ASCII logo, tagline, press Enter to start
// - render_selection_screen: Category selection with list + preview panel
// - render_summary_screen: Full review of all selections before scaffolding
// - render_progress_screen: Animated progress during template scaffolding
// - render_complete_screen: Success message with next steps
// =============================================================================

use super::{theme, widgets};
use crate::core::selections::{category_display_name, category_instruction, UserSelections};
use crate::config::constants;
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::Modifier,
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Wrap, Row, Table, Cell},
    Frame,
};

/// Render the welcome screen with ASCII logo and instructions
pub fn render_welcome_screen(frame: &mut Frame, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(2),   // top padding
            Constraint::Length(8),   // logo
            Constraint::Length(2),   // tagline
            Constraint::Length(1),   // version
            Constraint::Min(2),      // spacer
            Constraint::Length(3),   // instructions
            Constraint::Length(1),   // bottom padding
        ])
        .split(area);

    // Logo
    let logo = Paragraph::new(constants::LOGO)
        .style(theme::logo_style())
        .alignment(Alignment::Center);
    frame.render_widget(logo, chunks[1]);

    // Tagline
    let tagline = Paragraph::new(constants::TAGLINE)
        .style(theme::heading_style())
        .alignment(Alignment::Center);
    frame.render_widget(tagline, chunks[2]);

    // Version
    let version = Paragraph::new(format!("v{}", constants::VERSION))
        .style(theme::muted_style())
        .alignment(Alignment::Center);
    frame.render_widget(version, chunks[3]);

    // Instructions
    let instructions = Paragraph::new(vec![
        Line::from(vec![
            Span::styled("Press ", theme::body_style()),
            Span::styled(
                "Enter",
                theme::keyhint_style().add_modifier(Modifier::BOLD),
            ),
            Span::styled(" to start the wizard", theme::body_style()),
        ]),
        Line::from(vec![
            Span::styled("Press ", theme::body_style()),
            Span::styled(
                "q",
                theme::keyhint_style().add_modifier(Modifier::BOLD),
            ),
            Span::styled(" to quit", theme::body_style()),
        ]),
    ])
    .alignment(Alignment::Center);
    frame.render_widget(instructions, chunks[5]);
}

/// Render a category selection screen with list on the left and preview on the right
pub fn render_selection_screen(
    frame: &mut Frame,
    area: Rect,
    category: &str,
    list_state: &mut widgets::SelectionListState,
    step_current: usize,
    step_total: usize,
) {
    // Main layout: header, content, footer
    let main_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // header with step indicator
            Constraint::Min(10),   // content area
            Constraint::Length(2), // key hints
        ])
        .split(area);

    // Header: step indicator and category name
    let display_name = category_display_name(category);
    let instruction = category_instruction(category);
    let header = Paragraph::new(vec![
        Line::from(vec![
            Span::styled(
                format!(" Step {}/{} ", step_current, step_total),
                theme::keyhint_style().add_modifier(Modifier::BOLD),
            ),
            Span::styled(format!(" — {} ", display_name), theme::heading_style()),
        ]),
        Line::from(Span::styled(
            format!(" {}", instruction),
            theme::muted_style(),
        )),
    ])
    .block(
        Block::default()
            .borders(Borders::BOTTOM)
            .border_style(theme::inactive_border_style()),
    );
    frame.render_widget(header, main_chunks[0]);

    // Content: selection list (left) + preview panel (right)
    let content_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(50), // selection list
            Constraint::Percentage(50), // preview panel
        ])
        .split(main_chunks[1]);

    // Render selection list
    widgets::render_selection_list(
        frame,
        content_chunks[0],
        list_state,
        &display_name,
        true,
    );

    // Render preview panel with current item's description
    let preview_title = list_state.current_label().to_string();
    let preview_desc = list_state.current_description().to_string();
    widgets::render_preview_panel(
        frame,
        content_chunks[1],
        &preview_title,
        &preview_desc,
        false,
    );

    // Key hints
    let hints = match list_state.mode {
        crate::core::selections::SelectionMode::Single => {
            vec![
                ("↑↓", "Navigate"),
                ("Space", "Select"),
                ("Enter", "Confirm"),
                ("Esc", "Back"),
                ("q", "Quit"),
            ]
        }
        crate::core::selections::SelectionMode::OptionalSingle => {
            vec![
                ("↑↓", "Navigate"),
                ("Space", "Toggle"),
                ("Enter", "Confirm"),
                ("Esc", "Back"),
                ("q", "Quit"),
            ]
        }
        crate::core::selections::SelectionMode::Multi => {
            vec![
                ("↑↓", "Navigate"),
                ("Space", "Toggle"),
                ("a", "Select All"),
                ("Enter", "Confirm"),
                ("Esc", "Back"),
                ("q", "Quit"),
            ]
        }
    };
    widgets::render_key_hints(frame, main_chunks[2], &hints);
}

/// Render the summary screen showing all selections before scaffolding
pub fn render_summary_screen(
    frame: &mut Frame,
    area: Rect,
    selections: &UserSelections,
) {
    let main_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // header
            Constraint::Min(10),   // summary table
            Constraint::Length(4), // action prompt
            Constraint::Length(2), // key hints
        ])
        .split(area);

    // Header
    let header = Paragraph::new(vec![
        Line::from(Span::styled(
            " Project Summary ",
            theme::heading_style(),
        )),
        Line::from(Span::styled(
            format!(" Project: {} ", selections.project_name),
            theme::keyhint_style(),
        )),
    ])
    .block(
        Block::default()
            .borders(Borders::BOTTOM)
            .border_style(theme::inactive_border_style()),
    );
    frame.render_widget(header, main_chunks[0]);

    // Summary table
    let summary_lines = selections.summary_lines();
    let rows: Vec<Row> = summary_lines
        .iter()
        .map(|(category, value)| {
            Row::new(vec![
                Cell::from(Span::styled(
                    category.clone(),
                    theme::keyhint_style().add_modifier(Modifier::BOLD),
                )),
                Cell::from(Span::styled(value.clone(), theme::body_style())),
            ])
        })
        .collect();

    let table = Table::new(
        rows,
        [Constraint::Length(20), Constraint::Min(30)],
    )
    .block(
        Block::default()
            .title(" Your Selections ")
            .title_style(theme::heading_style())
            .borders(Borders::ALL)
            .border_style(theme::active_border_style()),
    )
    .column_spacing(3);

    frame.render_widget(table, main_chunks[1]);

    // Action prompt
    let prompt = Paragraph::new(vec![
        Line::from(""),
        Line::from(vec![
            Span::styled("  Package Manager: ", theme::muted_style()),
            Span::styled(
                "bun",
                theme::success_style(),
            ),
        ]),
        Line::from(vec![
            Span::styled(
                "  Ready to scaffold your project?",
                theme::heading_style(),
            ),
        ]),
    ]);
    frame.render_widget(prompt, main_chunks[2]);

    // Key hints
    let hints = vec![
        ("Enter", "Scaffold"),
        ("Esc", "Go Back"),
        ("q", "Quit"),
    ];
    widgets::render_key_hints(frame, main_chunks[3], &hints);
}

/// Render the progress screen during scaffolding
pub fn render_progress_screen(
    frame: &mut Frame,
    area: Rect,
    status_message: &str,
    progress_percent: u16,
    log_lines: &[String],
) {
    let main_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),  // header
            Constraint::Length(3),  // progress bar
            Constraint::Min(8),     // log output
            Constraint::Length(1),  // bottom padding
        ])
        .split(area);

    // Header
    let header = Paragraph::new(Line::from(Span::styled(
        " Scaffolding your project... ",
        theme::heading_style(),
    )))
    .block(
        Block::default()
            .borders(Borders::BOTTOM)
            .border_style(theme::inactive_border_style()),
    );
    frame.render_widget(header, main_chunks[0]);

    // Progress bar (rendered as a simple text gauge)
    let filled = (progress_percent as usize * 40) / 100;
    let empty = 40 - filled;
    let bar = format!(
        " [{}{}] {}% — {}",
        "█".repeat(filled),
        "░".repeat(empty),
        progress_percent,
        status_message,
    );
    let progress = Paragraph::new(Line::from(Span::styled(bar, theme::keyhint_style())))
        .block(Block::default().borders(Borders::ALL).border_style(theme::active_border_style()));
    frame.render_widget(progress, main_chunks[1]);

    // Log output
    let log_text: Vec<Line> = log_lines
        .iter()
        .map(|line| Line::from(Span::styled(line.clone(), theme::muted_style())))
        .collect();
    let log = Paragraph::new(log_text)
        .block(
            Block::default()
                .title(" Log ")
                .title_style(theme::heading_style())
                .borders(Borders::ALL)
                .border_style(theme::inactive_border_style()),
        )
        .wrap(Wrap { trim: true });
    frame.render_widget(log, main_chunks[2]);
}

/// Render the completion screen with success message and next steps
pub fn render_complete_screen(
    frame: &mut Frame,
    area: Rect,
    project_name: &str,
    project_path: &str,
) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(2),  // top padding
            Constraint::Length(3),  // success message
            Constraint::Length(2),  // spacer
            Constraint::Min(10),    // next steps
            Constraint::Length(2),  // key hints
        ])
        .split(area);

    // Success message
    let success = Paragraph::new(vec![
        Line::from(Span::styled(
            " ✓ Project scaffolded successfully! ",
            theme::success_style(),
        )),
        Line::from(Span::styled(
            format!("   {} → {}", project_name, project_path),
            theme::body_style(),
        )),
    ]);
    frame.render_widget(success, chunks[1]);

    // Next steps
    let next_steps = Paragraph::new(vec![
        Line::from(Span::styled(" Next Steps:", theme::heading_style())),
        Line::from(""),
        Line::from(vec![
            Span::styled("  1. ", theme::keyhint_style()),
            Span::styled(format!("cd {}", project_name), theme::body_style()),
        ]),
        Line::from(vec![
            Span::styled("  2. ", theme::keyhint_style()),
            Span::styled("Copy .env.example to .env and fill in your keys", theme::body_style()),
        ]),
        Line::from(vec![
            Span::styled("  3. ", theme::keyhint_style()),
            Span::styled("bun install", theme::body_style()),
        ]),
        Line::from(vec![
            Span::styled("  4. ", theme::keyhint_style()),
            Span::styled("bun dev", theme::body_style()),
        ]),
        Line::from(""),
        Line::from(Span::styled(
            "  Your selections are saved in weave.toml for reproducibility.",
            theme::muted_style(),
        )),
    ])
    .block(
        Block::default()
            .title(" Getting Started ")
            .title_style(theme::heading_style())
            .borders(Borders::ALL)
            .border_style(theme::active_border_style()),
    );
    frame.render_widget(next_steps, chunks[3]);

    // Key hints
    let hints = vec![("Enter", "Exit"), ("q", "Quit")];
    widgets::render_key_hints(frame, chunks[4], &hints);
}

/// Helper to create a centered rect within a parent area
pub fn centered_rect(percent_x: u16, percent_y: u16, area: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(area);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}
