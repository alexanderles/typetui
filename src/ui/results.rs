//! Test results screen: sidebar, WPM chart, stat strip, footer layout.

use ratatui::{
    layout::{Alignment, Constraint, Layout, Rect},
    style::{Color, Modifier, Style},
    symbols,
    text::{Line, Span},
    widgets::{Axis, Block, Borders, Chart, Dataset, GraphType, Padding, Paragraph},
    Frame,
};

use crate::app::{App, Stats, TestMode, WORD_OPTIONS};

use super::{styles, theme, title};

/// Horizontal and vertical padding inside the chart border (ratatui `Block` padding).
const CHART_INNER_PADDING: Padding = Padding::symmetric(2, 1);

pub(crate) fn draw_results_screen(frame: &mut Frame, app: &App) {
    let area = frame.area();
    let stats = app.calculate_stats();

    let h = area.height;
    let footer_h: u16 = 1;
    let title_block: u16 = 3 + 8 + 1; // padding, ASCII title, gap under title
    let min_content: u16 = 10;
    let mut blank_h = (h / 4).max(1);
    let mut content_h = h.saturating_sub(title_block + blank_h + footer_h);
    if content_h < min_content {
        blank_h = h.saturating_sub(title_block + footer_h + min_content);
        content_h = min_content;
    }

    let top_chunks = Layout::vertical([
        Constraint::Length(3),
        Constraint::Length(8),
        Constraint::Length(1),
    ])
    .split(area);

    title::draw_title(frame, top_chunks[1]);

    let content_y = area.y.saturating_add(title_block);
    let content_rect = Rect::new(area.x, content_y, area.width, content_h);

    let footer_y = content_y.saturating_add(content_h).saturating_add(blank_h);
    let footer_rect = Rect::new(area.x, footer_y, area.width, footer_h);

    let content_rows = Layout::vertical([
        Constraint::Min(6),
        Constraint::Length(3),
    ])
    .split(content_rect);

    let main_row = content_rows[0];
    let use_horizontal = main_row.width >= 52 && main_row.height >= 5;

    if use_horizontal {
        let cols = Layout::horizontal([
            Constraint::Length(24),
            Constraint::Min(8),
        ])
        .split(main_row);
        draw_results_sidebar(frame, app, &stats, cols[0], true);
        draw_results_wpm_chart(frame, app, cols[1]);
    } else {
        let rows = Layout::vertical([
            Constraint::Length(11),
            Constraint::Min(5),
        ])
        .split(main_row);
        draw_results_sidebar(frame, app, &stats, rows[0], true);
        draw_results_wpm_chart(frame, app, rows[1]);
    }

    draw_results_stats_strip(frame, app, &stats, content_rows[1]);

    let instructions = Paragraph::new(Line::from(Span::styled(
        "Press 'r' to return to menu | Press 'q' to quit",
        styles::label(),
    )))
    .alignment(Alignment::Center);

    frame.render_widget(instructions, footer_rect);
}

fn draw_results_sidebar(
    frame: &mut Frame,
    app: &App,
    stats: &Stats,
    area: Rect,
    compact: bool,
) {
    let word_count = WORD_OPTIONS
        .get(app.word_option_idx)
        .copied()
        .unwrap_or(25);
    let test_mode_line = match app.test_mode {
        TestMode::Time => format!("time {}", app.duration.as_secs()),
        TestMode::Words => format!("words {}", word_count),
    };

    let gap = || Line::from("");
    let tight_gap = || Line::from(Span::raw(" "));

    let mut lines = vec![
        Line::from(Span::styled("wpm", Style::default().fg(theme::LABEL))),
        Line::from(Span::styled(
            format!("{:.0}", stats.wpm),
            Style::default()
                .fg(theme::PRIMARY)
                .add_modifier(Modifier::BOLD),
        )),
    ];
    if compact {
        lines.push(tight_gap());
    } else {
        lines.push(gap());
    }
    lines.extend([
        Line::from(Span::styled("acc", Style::default().fg(theme::LABEL))),
        Line::from(Span::styled(
            format!("{:.0}%", stats.accuracy),
            Style::default()
                .fg(theme::PRIMARY)
                .add_modifier(Modifier::BOLD),
        )),
    ]);
    if compact {
        lines.push(tight_gap());
    } else {
        lines.push(gap());
    }
    lines.push(stat_line(
        "errors ",
        format!("{}", stats.total_errors),
        Color::Red,
    ));
    lines.push(stat_line(
        "words with errors ",
        format!("{}", stats.words_with_errors),
        Color::LightRed,
    ));
    lines.push(Line::from(vec![
        Span::styled("chars ", Style::default().fg(theme::LABEL)),
        Span::styled(
            format!("{}", stats.correct_chars),
            Style::default().fg(Color::Green).add_modifier(Modifier::BOLD),
        ),
        Span::styled("/", Style::default().fg(theme::LABEL)),
        Span::styled(
            format!("{}", stats.total_chars_typed),
            Style::default().add_modifier(Modifier::BOLD),
        ),
    ]));
    if compact {
        lines.push(tight_gap());
    } else {
        lines.push(gap());
    }
    lines.push(Line::from(Span::styled("test type", Style::default().fg(theme::LABEL))));
    lines.push(Line::from(Span::styled(
        test_mode_line,
        Style::default().fg(theme::PRIMARY),
    )));
    lines.push(Line::from(Span::styled("english", Style::default().fg(theme::PRIMARY))));
    if app.punctuation {
        lines.push(Line::from(Span::styled(
            "punctuation",
            Style::default().fg(theme::PRIMARY),
        )));
    }

    let sidebar = Paragraph::new(lines)
        .alignment(Alignment::Left)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .padding(Padding::left(2))
                .title(" Results ")
                .title_alignment(Alignment::Left),
        );

    frame.render_widget(sidebar, area);
}

fn draw_results_stats_strip(frame: &mut Frame, app: &App, stats: &Stats, area: Rect) {
    let consistency = wpm_series_consistency(&app.wpm_chart_points);
    let time_s = match app.test_mode {
        TestMode::Time => app.duration.as_secs(),
        TestMode::Words => app.time_elapsed().as_secs_f64().round() as u64,
    };

    let strip_style = Style::default().fg(theme::LABEL);
    let strip_block = Block::default()
        .borders(Borders::TOP)
        .border_style(strip_style);
    let inner = strip_block.inner(area);
    frame.render_widget(strip_block, area);

    let cols = Layout::horizontal([
        Constraint::Ratio(1, 4),
        Constraint::Ratio(1, 4),
        Constraint::Ratio(1, 4),
        Constraint::Ratio(1, 4),
    ])
    .split(inner);

    let raw_val = format!("{:.0}", stats.wpm);
    let chars_val = format!("{}/{}", stats.correct_chars, stats.total_chars_typed);
    let cons_val = format!("{:.0}%", consistency);
    let time_val = format!("{}s", time_s);

    let blocks = [
        ("raw wpm", raw_val.as_str()),
        ("characters typed", chars_val.as_str()),
        ("consistency", cons_val.as_str()),
        ("time elapsed", time_val.as_str()),
    ];
    for (i, (label, value)) in blocks.iter().enumerate() {
        let cell = Paragraph::new(vec![
            Line::from(Span::styled(*label, strip_style)),
            Line::from(Span::styled(
                *value,
                Style::default()
                    .fg(theme::PRIMARY)
                    .add_modifier(Modifier::BOLD),
            )),
        ])
        .alignment(Alignment::Center);
        frame.render_widget(cell, cols[i]);
    }
}

fn wpm_series_consistency(chart: &[(f64, f64)]) -> f64 {
    if chart.len() < 2 {
        return 100.0;
    }
    let values: Vec<f64> = chart.iter().map(|(_, y)| *y).collect();
    let n = values.len() as f64;
    let mean = values.iter().sum::<f64>() / n;
    if mean < 1.0 {
        return 100.0;
    }
    let variance = values.iter().map(|v| (v - mean).powi(2)).sum::<f64>() / n;
    let cv = variance.sqrt() / mean;
    ((1.0 - cv.min(1.0)) * 100.0).clamp(0.0, 100.0)
}

fn draw_results_wpm_chart(frame: &mut Frame, app: &App, area: Rect) {
    if app.wpm_chart_points.is_empty() {
        let empty = Paragraph::new(Line::from(Span::styled(
            "No WPM history (sub-second test or no keystrokes).",
            styles::label(),
        )))
        .alignment(Alignment::Center)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .padding(CHART_INNER_PADDING)
                .title(" WPM ")
                .title_alignment(Alignment::Left),
        );
        frame.render_widget(empty, area);
        return;
    }

    let last_x = app
        .wpm_chart_points
        .iter()
        .map(|(x, _)| *x)
        .fold(0.0f64, f64::max);
    let x_max = match app.test_mode {
        TestMode::Time => app.duration.as_secs_f64().max(last_x).max(1.0),
        TestMode::Words => last_x.max(1.0),
    };

    let max_wpm = app
        .wpm_chart_points
        .iter()
        .map(|(_, y)| *y)
        .fold(0.0f64, f64::max);
    let y_max = ((max_wpm * 1.15 / 10.0).ceil() * 10.0).max(20.0);

    let datasets = vec![Dataset::default()
        .marker(symbols::Marker::Braille)
        .graph_type(GraphType::Line)
        .style(Style::default().fg(theme::PRIMARY))
        .data(app.wpm_chart_points.as_slice())];

    let x_mid = x_max / 2.0;
    let y_mid = y_max / 2.0;

    let chart = Chart::new(datasets)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .padding(CHART_INNER_PADDING)
                .title(" WPM ")
                .title_alignment(Alignment::Left),
        )
        .legend_position(None)
        .x_axis(
            Axis::default()
                .title(Line::from(Span::styled(
                    "Time (s)",
                    Style::default().fg(theme::LABEL),
                )))
                .style(Style::default().fg(theme::LABEL))
                .bounds([0.0, x_max])
                .labels([
                    Span::raw("0"),
                    Span::styled(
                        format!("{:.0}", x_mid),
                        Style::default().fg(theme::LABEL),
                    ),
                    Span::styled(
                        format!("{:.0}", x_max),
                        Style::default().fg(theme::LABEL),
                    ),
                ]),
        )
        .y_axis(
            Axis::default()
                .title(Line::from(Span::styled(
                    "WPM",
                    Style::default().fg(theme::LABEL),
                )))
                .style(Style::default().fg(theme::LABEL))
                .bounds([0.0, y_max])
                .labels([
                    Span::raw("0"),
                    Span::styled(
                        format!("{:.0}", y_mid),
                        Style::default().fg(theme::LABEL),
                    ),
                    Span::styled(
                        format!("{:.0}", y_max),
                        Style::default().fg(theme::LABEL),
                    ),
                ]),
        );

    frame.render_widget(chart, area);
}

fn stat_line(label: &str, value: String, color: Color) -> Line<'static> {
    Line::from(vec![
        Span::raw(label.to_string()),
        Span::styled(
            value,
            Style::default().fg(color).add_modifier(Modifier::BOLD),
        ),
    ])
}
