//! Reusable [`Style`](ratatui::style::Style) helpers.

use ratatui::style::{Color, Modifier, Style};

use super::theme;

/// Style for selected/highlighted menu items.
pub(crate) fn selected() -> Style {
    Style::default()
        .fg(theme::PRIMARY)
        .add_modifier(Modifier::BOLD)
}

/// Style for unselected menu items.
pub(crate) fn unselected() -> Style {
    Style::default().fg(Color::White)
}

/// Style for labels and help text.
pub(crate) fn label() -> Style {
    Style::default().fg(Color::DarkGray)
}

/// Style for correctly typed characters.
pub(crate) fn correct() -> Style {
    Style::default().fg(Color::Green)
}

/// Style for incorrectly typed characters.
pub(crate) fn incorrect() -> Style {
    Style::default().fg(Color::Red)
}

/// Style for characters that were corrected (initially incorrect, then fixed).
pub(crate) fn corrected() -> Style {
    Style::default().fg(Color::LightYellow)
}

/// Style for characters not yet typed.
pub(crate) fn pending() -> Style {
    Style::default().fg(Color::DarkGray)
}

/// Style for the cursor position (current character to type).
pub(crate) fn cursor() -> Style {
    Style::default()
        .fg(Color::White)
        .add_modifier(Modifier::UNDERLINED)
}
