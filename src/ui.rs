//! User interface rendering for the typing test.
//!
//! - [`menu`](crate::ui::menu): configuration screen
//! - [`typing`](crate::ui::typing): active test (timer, words, input)
//! - [`results`](crate::ui::results): summary, chart, and stat strip
//!
//! Shared pieces: [`theme`](crate::ui::theme) (colors), [`styles`](crate::ui::styles) (ratatui
//! [`Style`](ratatui::style::Style) helpers), [`title`](crate::ui::title) (TYPETUI ASCII art).

mod menu;
mod results;
mod styles;
mod theme;
mod title;
mod typing;

use ratatui::Frame;

use crate::app::{App, CurrentScreen};

/// Main entry point for rendering the UI.
///
/// Dispatches to the appropriate screen based on app state.
pub fn draw(frame: &mut Frame, app: &App) {
    match &app.state {
        CurrentScreen::Menu(_) => menu::draw_menu_screen(frame, app),
        CurrentScreen::TypingTest(_) => typing::draw_typing_screen(frame, app),
        CurrentScreen::TestResults => results::draw_results_screen(frame, app),
    }
}
