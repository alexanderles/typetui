//! Menu screen: title, settings box, help.

use ratatui::{
    layout::{Alignment, Constraint, Layout, Rect},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

use crate::app::{App, CurrentScreen, MenuField, TestMode, TIME_OPTIONS, WORD_OPTIONS};

use super::{styles, title};

pub(crate) fn draw_menu_screen(frame: &mut Frame, app: &App) {
    let area = frame.area();

    let chunks = Layout::vertical([
        Constraint::Length(3),  // Top padding
        Constraint::Length(8),  // ASCII art title
        Constraint::Length(2),  // Spacing below title
        Constraint::Length(11), // Settings box
        Constraint::Min(1),     // Spacer
        Constraint::Length(2),  // Help text
    ])
    .split(area);

    title::draw_title(frame, chunks[1]);
    draw_settings(frame, app, area, chunks[3]);
    draw_menu_help(frame, chunks[5]);
}

fn draw_settings(frame: &mut Frame, app: &App, full_area: Rect, chunk: Rect) {
    let settings_width = 40u16;
    let settings_x = full_area.width.saturating_sub(settings_width) / 2;
    let settings_area = Rect::new(
        settings_x,
        chunk.y,
        settings_width.min(full_area.width),
        chunk.height,
    );

    let settings_block = Block::default()
        .borders(Borders::ALL)
        .border_style(styles::label());
    frame.render_widget(settings_block, settings_area);

    let inner_area = Rect::new(
        settings_area.x + 2,
        settings_area.y + 1,
        settings_area.width.saturating_sub(4),
        settings_area.height.saturating_sub(2),
    );

    let rows = Layout::vertical([
        Constraint::Length(1), // Spacer
        Constraint::Length(1), // Mode
        Constraint::Length(1), // Spacer
        Constraint::Length(1), // Value
        Constraint::Length(1), // Spacer
        Constraint::Length(1), // Punctuation
        Constraint::Length(1), // Spacer
        Constraint::Length(1), // Start
    ])
    .split(inner_area);

    draw_mode_row(frame, app, rows[1]);
    draw_value_row(frame, app, rows[3]);
    draw_punctuation_row(frame, app, rows[5]);
    draw_start_row(frame, app, rows[7]);
}

fn draw_mode_row(frame: &mut Frame, app: &App, area: Rect) {
    let is_selected = if let CurrentScreen::Menu(menu) = &app.state {
        menu.menu_field == MenuField::Mode
    } else {
        false
    };
    let style = if is_selected {
        styles::selected()
    } else {
        styles::unselected()
    };

    let mode_value = match app.test_mode {
        TestMode::Time => "Time",
        TestMode::Words => "Words",
    };

    let line = Line::from(vec![
        Span::styled("    Mode:  ", styles::label()),
        Span::styled(format!("◄ {:^5} ►", mode_value), style),
    ]);

    frame.render_widget(Paragraph::new(line), area);
}

fn draw_value_row(frame: &mut Frame, app: &App, area: Rect) {
    let is_selected = if let CurrentScreen::Menu(menu) = &app.state {
        menu.menu_field == MenuField::Value
    } else {
        false
    };
    let style = if is_selected {
        styles::selected()
    } else {
        styles::unselected()
    };

    let (label, value_text) = match app.test_mode {
        TestMode::Time => {
            let secs = app.time_seconds();
            let display = if secs >= 60 {
                format!("{}m {}s", secs / 60, secs % 60)
            } else {
                format!("{}s", secs)
            };
            let time_option_idx = if let CurrentScreen::Menu(menu) = &app.state {
                menu.time_option_idx
            } else {
                0
            };
            let left = if time_option_idx > 0 { "◄ " } else { "  " };
            let right = if time_option_idx < TIME_OPTIONS.len() - 1 {
                " ►"
            } else {
                "  "
            };
            ("    Time:", format!("{}{:^7}{}", left, display, right))
        }
        TestMode::Words => {
            let count = app.target_word_count();
            let word_option_idx = if let CurrentScreen::Menu(menu) = &app.state {
                menu.word_option_idx
            } else {
                0
            };
            let left = if word_option_idx > 0 { "◄ " } else { "  " };
            let right = if word_option_idx < WORD_OPTIONS.len() - 1 {
                " ►"
            } else {
                "  "
            };
            ("   Words:", format!("{}{:^7}{}", left, count, right))
        }
    };

    let line = Line::from(vec![
        Span::styled(format!("{}  ", label), styles::label()),
        Span::styled(value_text, style),
    ]);

    frame.render_widget(Paragraph::new(line), area);
}

fn draw_punctuation_row(frame: &mut Frame, app: &App, area: Rect) {
    let is_selected = if let CurrentScreen::Menu(menu) = &app.state {
        menu.menu_field == MenuField::Punctuation
    } else {
        false
    };
    let style = if is_selected {
        styles::selected()
    } else {
        styles::unselected()
    };

    let value = if app.punctuation { "On" } else { "Off" };

    let line = Line::from(vec![
        Span::styled("   Punct:  ", styles::label()),
        Span::styled(format!("◄ {:^5} ►", value), style),
    ]);

    frame.render_widget(Paragraph::new(line), area);
}

fn draw_start_row(frame: &mut Frame, app: &App, area: Rect) {
    let is_selected = if let CurrentScreen::Menu(menu) = &app.state {
        menu.menu_field == MenuField::Start
    } else {
        false
    };
    let style = if is_selected {
        styles::selected()
    } else {
        styles::unselected()
    };

    let line = Line::from(vec![
        Span::raw("         "),
        Span::styled("[ Start ]", style),
    ]);

    frame.render_widget(Paragraph::new(line), area);
}

fn draw_menu_help(frame: &mut Frame, area: Rect) {
    let help = Paragraph::new(Line::from(Span::styled(
        "↑↓/jk: navigate  ←→/hl: change  Enter: start  q: quit",
        styles::label(),
    )))
    .alignment(Alignment::Center);

    frame.render_widget(help, area);
}
