// =============================================================================
// Theme - Color palette and styling constants for the Ratatui terminal UI
// =============================================================================
//
// Table of Contents:
// - Color constants for the Weave brand
// - Style builders for common UI elements
// =============================================================================

use ratatui::style::{Color, Modifier, Style};

/// Primary brand color — electric blue
pub const PRIMARY: Color = Color::Rgb(99, 102, 241);

/// Secondary accent — cyan/teal
pub const SECONDARY: Color = Color::Rgb(34, 211, 238);

/// Success / confirmed — green
pub const SUCCESS: Color = Color::Rgb(34, 197, 94);

/// Warning — amber
pub const WARNING: Color = Color::Rgb(251, 191, 36);

/// Error / destructive — red
pub const ERROR: Color = Color::Rgb(239, 68, 68);

/// Muted text — gray
pub const MUTED: Color = Color::Rgb(148, 163, 184);

/// Background highlight for selected items
pub const HIGHLIGHT_BACKGROUND: Color = Color::Rgb(30, 41, 59);

/// Bright white for headings
pub const HEADING: Color = Color::Rgb(248, 250, 252);

/// Dimmed border color
pub const BORDER: Color = Color::Rgb(71, 85, 105);

/// Active border color (focused panel)
pub const BORDER_ACTIVE: Color = Color::Rgb(99, 102, 241);

/// Logo color — gradient simulated with a single bright color
pub const LOGO: Color = Color::Rgb(139, 92, 246);

// -- Style Builders ----------------------------------------------------------

/// Style for the main heading text
pub fn heading_style() -> Style {
    Style::default().fg(HEADING).add_modifier(Modifier::BOLD)
}

/// Style for normal body text
pub fn body_style() -> Style {
    Style::default().fg(Color::Rgb(226, 232, 240))
}

/// Style for muted/secondary text
pub fn muted_style() -> Style {
    Style::default().fg(MUTED)
}

/// Style for a selected/highlighted list item
pub fn selected_style() -> Style {
    Style::default()
        .fg(PRIMARY)
        .bg(HIGHLIGHT_BACKGROUND)
        .add_modifier(Modifier::BOLD)
}

/// Style for a checked/enabled item in a multi-select
pub fn checked_style() -> Style {
    Style::default().fg(SUCCESS)
}

/// Style for the currently focused border
pub fn active_border_style() -> Style {
    Style::default().fg(BORDER_ACTIVE)
}

/// Style for unfocused borders
pub fn inactive_border_style() -> Style {
    Style::default().fg(BORDER)
}

/// Style for key hint text (for example, "[Enter] Confirm")
pub fn keyhint_style() -> Style {
    Style::default().fg(SECONDARY)
}

/// Style for error messages
pub fn error_style() -> Style {
    Style::default().fg(ERROR).add_modifier(Modifier::BOLD)
}

/// Style for warning messages
pub fn warning_style() -> Style {
    Style::default().fg(WARNING)
}

/// Style for success messages
pub fn success_style() -> Style {
    Style::default().fg(SUCCESS).add_modifier(Modifier::BOLD)
}

/// Style for the ASCII logo
pub fn logo_style() -> Style {
    Style::default().fg(LOGO).add_modifier(Modifier::BOLD)
}
