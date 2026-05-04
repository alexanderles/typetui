//! Active typing test screen: timer, word list, input.

use ratatui::{
    layout::{Alignment, Constraint, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

use crate::app::{App, CharResult, CurrentScreen, TestMode};

use super::styles;

pub(crate) fn draw_typing_screen(frame: &mut Frame, app: &App) {
    let area = frame.area();

    let horizontal_padding = 8u16;
    let padded_area = Rect::new(
        area.x + horizontal_padding,
        area.y,
        area.width.saturating_sub(horizontal_padding * 2),
        area.height,
    );

    let chunks = Layout::vertical([
        Constraint::Length(3), // Timer
        Constraint::Length(3), // Top padding
        Constraint::Length(5), // Words (3 lines + border)
        Constraint::Length(3), // Bottom padding
        Constraint::Length(3), // Input
        Constraint::Min(0),    // Absorb remaining space
    ])
    .split(padded_area);

    draw_timer(frame, app, chunks[0]);
    draw_words(frame, app, chunks[2]);
    draw_input(frame, app, chunks[4]);
}

fn draw_timer(frame: &mut Frame, app: &App, area: Rect) {
    let (text, color, title) = if app.start_time.is_none() {
        ("Type to start...".to_string(), Color::Green, "Timer")
    } else {
        match app.test_mode {
            TestMode::Time => {
                let remaining = app.time_remaining().as_secs_f64();
                let color = if remaining <= 5.0 {
                    Color::Red
                } else if remaining <= 10.0 {
                    Color::Yellow
                } else {
                    Color::Green
                };
                (format!("{:.1}s", remaining), color, "Timer")
            }
            TestMode::Words => {
                let elapsed = app.time_elapsed().as_secs_f64();
                let current_word_idx = if let CurrentScreen::TypingTest(test_state) = &app.state {
                    test_state.current_word_idx
                } else {
                    0
                };
                let text = format!(
                    "{}/{} words  |  {:.1}s",
                    current_word_idx,
                    app.word_states.len(),
                    elapsed
                );
                (text, Color::Cyan, "Progress")
            }
        }
    };

    let timer = Paragraph::new(text)
        .style(Style::default().fg(color).add_modifier(Modifier::BOLD))
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::ALL).title(title));

    frame.render_widget(timer, area);
}

/// Renders the words to type with scrolling support.
///
/// Shows 3 lines at a time, scrolling when the user reaches line 2.
fn draw_words(frame: &mut Frame, app: &App, area: Rect) {
    let block = Block::default().borders(Borders::ALL).title("Words");
    let inner_area = block.inner(area);
    frame.render_widget(block, area);

    let side_padding = 4u16;
    let padded_inner = Rect::new(
        inner_area.x + side_padding,
        inner_area.y,
        inner_area.width.saturating_sub(side_padding * 2),
        inner_area.height,
    );

    let line_width = padded_inner.width as usize;
    if line_width == 0 {
        return;
    }

    let lines = build_word_lines(app, line_width);
    let current_word_idx = if let CurrentScreen::TypingTest(test_state) = &app.state {
        test_state.current_word_idx
    } else {
        0
    };
    let current_line = find_current_line(&lines, current_word_idx);
    let first_visible = current_line.saturating_sub(1).max(if current_line >= 2 {
        current_line - 1
    } else {
        0
    });

    let visible_lines: Vec<Line> = lines
        .iter()
        .skip(first_visible)
        .take(3)
        .map(|line_words| render_word_line(app, line_words))
        .collect();

    frame.render_widget(Paragraph::new(visible_lines), padded_inner);
}

/// Builds lines of words based on available width.
///
/// Returns a vector of lines, where each line is a vector of (word_index, word) tuples.
fn build_word_lines(app: &App, line_width: usize) -> Vec<Vec<(usize, String)>> {
    let mut lines = Vec::new();
    let mut current_line = Vec::new();
    let mut current_width = 0;

    for (idx, word_state) in app.word_states.iter().enumerate() {
        let word_len = word_state.target.len() + 1; // +1 for space

        if current_width + word_len > line_width && !current_line.is_empty() {
            lines.push(current_line);
            current_line = Vec::new();
            current_width = 0;
        }

        current_line.push((idx, word_state.target.clone()));
        current_width += word_len;
    }

    if !current_line.is_empty() {
        lines.push(current_line);
    }

    lines
}

fn find_current_line(lines: &[Vec<(usize, String)>], current_word_idx: usize) -> usize {
    lines
        .iter()
        .position(|line| line.iter().any(|(idx, _)| *idx == current_word_idx))
        .unwrap_or(0)
}

fn render_word_line(app: &App, line_words: &[(usize, String)]) -> Line<'static> {
    let mut spans = Vec::new();

    let current_word_idx = if let CurrentScreen::TypingTest(test_state) = &app.state {
        test_state.current_word_idx
    } else {
        0
    };

    let space_is_cursor = if let CurrentScreen::TypingTest(test_state) = &app.state {
        if current_word_idx < app.word_states.len() {
            let word_state = &app.word_states[current_word_idx];
            test_state.typed_input.len() >= word_state.target.len()
        } else {
            false
        }
    } else {
        false
    };

    for (idx, word) in line_words {
        if *idx < current_word_idx {
            spans.extend(render_word_characters(app, *idx, false));
            spans.push(Span::raw(" "));
        } else if *idx == current_word_idx {
            spans.extend(render_current_word(app, *idx));
            let space_style = if space_is_cursor {
                styles::cursor()
            } else {
                Style::default()
            };
            spans.push(Span::styled(" ", space_style));
        } else {
            spans.push(Span::styled(word.clone(), styles::pending()));
            spans.push(Span::raw(" "));
        }
    }

    Line::from(spans)
}

fn render_word_characters(app: &App, word_idx: usize, is_current: bool) -> Vec<Span<'static>> {
    let mut spans = Vec::new();

    if word_idx >= app.word_states.len() {
        return spans;
    }

    let word_state = &app.word_states[word_idx];
    let typed_len = if is_current {
        if let CurrentScreen::TypingTest(test_state) = &app.state {
            test_state.typed_input.len()
        } else {
            0
        }
    } else {
        word_state.typed.len()
    };

    for (char_idx, c) in word_state.target.chars().enumerate() {
        let style = if char_idx < typed_len {
            match word_state.results.get(char_idx) {
                Some(CharResult::Correct) => styles::correct(),
                Some(CharResult::Incorrect) => styles::incorrect(),
                Some(CharResult::Corrected) => styles::corrected(),
                None => styles::pending(),
            }
        } else if is_current && char_idx == typed_len {
            styles::cursor()
        } else {
            styles::pending()
        };
        spans.push(Span::styled(c.to_string(), style));
    }

    if typed_len > word_state.target.len() {
        let extra: String = if is_current {
            if let CurrentScreen::TypingTest(test_state) = &app.state {
                test_state
                    .typed_input
                    .chars()
                    .skip(word_state.target.len())
                    .collect()
            } else {
                String::new()
            }
        } else {
            word_state
                .typed
                .chars()
                .skip(word_state.target.len())
                .collect()
        };
        if !extra.is_empty() {
            spans.push(Span::styled(extra, styles::incorrect()));
        }
    }

    spans
}

fn render_current_word(app: &App, word_idx: usize) -> Vec<Span<'static>> {
    render_word_characters(app, word_idx, true)
}

fn draw_input(frame: &mut Frame, app: &App, area: Rect) {
    let typed_input = if let CurrentScreen::TypingTest(test_state) = &app.state {
        test_state.typed_input.as_str()
    } else {
        ""
    };
    let input = Paragraph::new(format!("> {}_", typed_input))
        .style(Style::default().fg(Color::White))
        .block(Block::default().borders(Borders::ALL).title("Input"));

    frame.render_widget(input, area);
}
