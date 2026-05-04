//! ASCII art title (TYPETUI) shared by menu and results screens.

use ratatui::{
    layout::{Alignment, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::Paragraph,
    Frame,
};

use super::theme;

/// ASCII art title displayed at the top of several screens.
const ASCII_TITLE: &[&str] = &[
    "████████╗██╗   ██╗██████╗ ███████╗████████╗██╗   ██╗██╗",
    "╚══██╔══╝╚██╗ ██╔╝██╔══██╗██╔════╝╚══██╔══╝██║   ██║██║",
    "   ██║    ╚████╔╝ ██████╔╝█████╗     ██║   ██║   ██║██║",
    "   ██║     ╚██╔╝  ██╔═══╝ ██╔══╝     ██║   ██║   ██║██║",
    "   ██║      ██║   ██║     ███████╗   ██║   ╚██████╔╝██║",
    "   ╚═╝      ╚═╝   ╚═╝     ╚══════╝   ╚═╝    ╚═════╝ ╚═╝",
];

pub(crate) fn draw_title(frame: &mut Frame, area: Rect) {
    let title_lines: Vec<Line> = ASCII_TITLE
        .iter()
        .map(|line| {
            Line::from(Span::styled(
                *line,
                Style::default()
                    .fg(theme::PRIMARY)
                    .add_modifier(Modifier::BOLD),
            ))
        })
        .collect();

    let title = Paragraph::new(title_lines).alignment(Alignment::Center);
    frame.render_widget(title, area);
}
