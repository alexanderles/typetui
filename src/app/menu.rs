//! Menu state and navigation logic.

use super::constants::{TIME_OPTIONS, WORD_OPTIONS};
use super::types::{MenuField, TestMode};

/// State for the menu screen.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MenuState {
    /// Currently selected menu field.
    pub menu_field: MenuField,
    /// Index into `TIME_OPTIONS` for selected duration.
    pub time_option_idx: usize,
    /// Index into `WORD_OPTIONS` for selected word count.
    pub word_option_idx: usize,
}

impl MenuState {
    /// Creates a new MenuState with default settings.
    pub fn new() -> Self {
        Self {
            menu_field: MenuField::Mode,
            time_option_idx: 1,
            word_option_idx: 1,
        }
    }

    /// Moves menu selection up.
    pub fn up(&mut self) {
        self.menu_field = match self.menu_field {
            MenuField::Mode => MenuField::Mode,
            MenuField::Value => MenuField::Mode,
            MenuField::Punctuation => MenuField::Value,
            MenuField::Start => MenuField::Punctuation,
        };
    }

    /// Moves menu selection down.
    pub fn down(&mut self) {
        self.menu_field = match self.menu_field {
            MenuField::Mode => MenuField::Value,
            MenuField::Value => MenuField::Punctuation,
            MenuField::Punctuation => MenuField::Start,
            MenuField::Start => MenuField::Start,
        };
    }

    /// Toggles between Time and Words mode.
    pub fn toggle_mode(&mut self, test_mode: &mut TestMode) {
        *test_mode = match *test_mode {
            TestMode::Time => TestMode::Words,
            TestMode::Words => TestMode::Time,
        };
    }

    /// Decrements the current value option (time or word count).
    pub fn decrement_value(&mut self, test_mode: TestMode) {
        match test_mode {
            TestMode::Time => {
                self.time_option_idx = self.time_option_idx.saturating_sub(1);
            }
            TestMode::Words => {
                self.word_option_idx = self.word_option_idx.saturating_sub(1);
            }
        }
    }

    /// Increments the current value option (time or word count).
    pub fn increment_value(&mut self, test_mode: TestMode) {
        match test_mode {
            TestMode::Time => {
                if self.time_option_idx < TIME_OPTIONS.len() - 1 {
                    self.time_option_idx += 1;
                }
            }
            TestMode::Words => {
                if self.word_option_idx < WORD_OPTIONS.len() - 1 {
                    self.word_option_idx += 1;
                }
            }
        }
    }

    /// Returns the selected time duration in seconds.
    pub fn time_seconds(&self) -> u32 {
        TIME_OPTIONS[self.time_option_idx]
    }

    /// Returns the selected target word count.
    pub fn target_word_count(&self) -> u32 {
        WORD_OPTIONS[self.word_option_idx]
    }
}

impl Default for MenuState {
    fn default() -> Self {
        Self::new()
    }
}
