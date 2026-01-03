//! Application state and logic for the typing test.

mod constants;
mod menu;
mod stats;
mod types;
mod typing_test;

use std::time::{Duration, Instant};

use crate::words::generate_words;

// Re-export types for convenience
pub use constants::{TIME_OPTIONS, WORD_OPTIONS};
pub use menu::MenuState;
pub use stats::Stats;
pub use types::{CharResult, CurrentScreen, MenuField, TestMode, WordState};
pub use typing_test::TypingTestState;

/// Main application state.
pub struct App {
    /// State for each word in the typing test.
    pub word_states: Vec<WordState>,
    /// Indices of words that were completed with errors.
    pub words_with_errors: Vec<usize>,
    /// Total characters typed across all words.
    pub total_chars_typed: usize,
    /// Total correctly typed characters.
    pub correct_chars: usize,
    /// When the test started (first character typed).
    pub start_time: Option<Instant>,
    /// When the test ended.
    pub end_time: Option<Instant>,
    /// Duration limit for timed mode.
    pub duration: Duration,
    /// Selected test mode.
    pub test_mode: TestMode,
    /// Index into `TIME_OPTIONS` for selected duration (persists between test runs).
    pub time_option_idx: usize,
    /// Index into `WORD_OPTIONS` for selected word count (persists between test runs).
    pub word_option_idx: usize,
    /// Current application state.
    pub state: CurrentScreen,
}

impl App {
    /// Creates a new App instance with default settings.
    pub fn new() -> Self {
        let time_option_idx = 1;
        let word_option_idx = 1;
        let mut menu = MenuState::new();
        menu.time_option_idx = time_option_idx;
        menu.word_option_idx = word_option_idx;

        Self {
            word_states: Vec::new(),
            words_with_errors: Vec::new(),
            total_chars_typed: 0,
            correct_chars: 0,
            start_time: None,
            end_time: None,
            duration: Duration::from_secs(TIME_OPTIONS[time_option_idx] as u64),
            test_mode: TestMode::Time,
            time_option_idx,
            word_option_idx,
            state: CurrentScreen::Menu(menu),
        }
    }

    /// Creates an App with predefined words for testing purposes.
    #[cfg(test)]
    pub fn with_words(words: Vec<String>) -> Self {
        let word_states = words
            .into_iter()
            .map(|target| WordState {
                target,
                typed: String::new(),
                results: Vec::new(),
            })
            .collect();
        Self {
            word_states,
            state: CurrentScreen::TypingTest(TypingTestState::new()),
            ..Self::new()
        }
    }

    /// Moves menu selection up.
    pub fn menu_up(&mut self) {
        if let CurrentScreen::Menu(ref mut menu) = &mut self.state {
            menu.up();
        }
    }

    /// Moves menu selection down.
    pub fn menu_down(&mut self) {
        if let CurrentScreen::Menu(ref mut menu) = &mut self.state {
            menu.down();
        }
    }

    /// Handles left arrow in menu (toggle mode or decrement value).
    pub fn menu_left(&mut self) {
        if let CurrentScreen::Menu(ref mut menu) = &mut self.state {
            match menu.menu_field {
                MenuField::Mode => {
                    menu.toggle_mode(&mut self.test_mode);
                }
                MenuField::Value => {
                    menu.decrement_value(self.test_mode);
                    // Update persisted indices
                    self.time_option_idx = menu.time_option_idx;
                    self.word_option_idx = menu.word_option_idx;
                }
                MenuField::Start => {}
            }
        }
    }

    /// Handles right arrow in menu (toggle mode or increment value).
    pub fn menu_right(&mut self) {
        if let CurrentScreen::Menu(ref mut menu) = &mut self.state {
            match menu.menu_field {
                MenuField::Mode => {
                    menu.toggle_mode(&mut self.test_mode);
                }
                MenuField::Value => {
                    menu.increment_value(self.test_mode);
                    // Update persisted indices
                    self.time_option_idx = menu.time_option_idx;
                    self.word_option_idx = menu.word_option_idx;
                }
                MenuField::Start => {}
            }
        }
    }

    /// Starts a new typing test with current settings.
    ///
    /// Generates random words and resets all test state.
    pub fn start_test(&mut self) {
        let word_count = match self.test_mode {
            TestMode::Time => 200,
            TestMode::Words => {
                if let CurrentScreen::Menu(ref menu) = &self.state {
                    menu.target_word_count() as usize
                } else {
                    25
                }
            }
        };

        let words = generate_words(word_count);
        self.word_states = words
            .into_iter()
            .map(|target| WordState {
                target,
                typed: String::new(),
                results: Vec::new(),
            })
            .collect();
        self.words_with_errors.clear();
        self.total_chars_typed = 0;
        self.correct_chars = 0;
        self.start_time = None;
        self.end_time = None;
        self.duration = Duration::from_secs(self.time_seconds() as u64);
        self.state = CurrentScreen::TypingTest(TypingTestState::new());
    }

    /// Resets the app back to the menu state.
    pub fn reset(&mut self) {
        self.word_states.clear();
        self.words_with_errors.clear();
        self.total_chars_typed = 0;
        self.correct_chars = 0;
        self.start_time = None;
        self.end_time = None;
        let mut menu = MenuState::new();
        // Restore persisted indices
        menu.time_option_idx = self.time_option_idx;
        menu.word_option_idx = self.word_option_idx;
        self.state = CurrentScreen::Menu(menu);
    }

    /// Called each frame to check for time-based test completion.
    pub fn tick(&mut self) {
        if let CurrentScreen::TypingTest(_) = &self.state {
            if self.test_mode == TestMode::Time {
                if let Some(start) = self.start_time {
                    if start.elapsed() >= self.duration {
                        self.end_time = Some(Instant::now());
                        self.state = CurrentScreen::TestResults;
                    }
                }
            }
        }
    }

    /// Handles a character being typed.
    ///
    /// - Starts the timer on first character
    /// - Records whether the character was correct or incorrect
    /// - Updates statistics
    /// - Ends the test in Words mode when the last letter of the last word is typed
    pub fn on_char(&mut self, c: char) {
        if let CurrentScreen::TypingTest(ref mut test_state) = &mut self.state {
            if test_state.current_word_idx < self.word_states.len() {
                let current_word_idx = test_state.current_word_idx;
                let word_state = &mut self.word_states[current_word_idx];

                test_state.on_char(
                    c,
                    word_state,
                    &mut self.total_chars_typed,
                    &mut self.correct_chars,
                    &mut self.start_time,
                );

                // Extract values needed for checking if test should end
                let typed_input_len = test_state.typed_input.len();
                let current_word_has_error = test_state.current_word_has_error;
                let typed_input = test_state.typed_input.clone();

                // Check if we should end the test (Words mode, last word, word is complete)
                if self.should_end_test(current_word_idx, typed_input_len) {
                    self.end_test(current_word_idx, &typed_input, current_word_has_error);
                }
            }
        }
    }

    /// Checks if the test should end.
    ///
    /// In Words mode, returns true when the last word is completed.
    fn should_end_test(&self, current_word_idx: usize, typed_input_len: usize) -> bool {
        if self.test_mode != TestMode::Words {
            return false;
        }

        let total_words = self.word_states.len();
        if current_word_idx >= total_words {
            return false;
        }

        let target_len = self.word_states[current_word_idx].target.len();
        current_word_idx == total_words - 1 && typed_input_len >= target_len
    }

    /// Ends the test, saving state and freezing the timer.
    fn end_test(
        &mut self,
        current_word_idx: usize,
        typed_input: &str,
        current_word_has_error: bool,
    ) {
        let word_state = &mut self.word_states[current_word_idx];
        let target_len = word_state.target.len();

        // Save typed input before ending
        word_state.typed = typed_input.to_string();

        // Check for errors
        let has_error = current_word_has_error || typed_input.len() > target_len;
        if has_error {
            self.words_with_errors.push(current_word_idx);
        }

        // Freeze the timer and end the test
        self.end_time = Some(Instant::now());
        self.state = CurrentScreen::TestResults;
    }

    /// Handles backspace being pressed.
    ///
    /// Removes the last typed character and its result.
    /// If at the start of the current word, goes back to the previous word.
    pub fn on_backspace(&mut self) {
        // Get current state info before mutable borrow
        let (should_go_back, prev_idx, current_word_idx, restored_input) =
            if let CurrentScreen::TypingTest(ref test_state) = &self.state {
                let should_go_back =
                    test_state.typed_input.is_empty() && test_state.current_word_idx > 0;
                let prev_idx = test_state.current_word_idx.saturating_sub(1);
                let current_word_idx = test_state.current_word_idx;
                let restored_input = if should_go_back && prev_idx < self.word_states.len() {
                    self.word_states[prev_idx].typed.clone()
                } else {
                    String::new()
                };
                (should_go_back, prev_idx, current_word_idx, restored_input)
            } else {
                (false, 0, 0, String::new())
            };

        if let CurrentScreen::TypingTest(ref mut test_state) = &mut self.state {
            // If at the start of current word and not at first word, go back to previous word
            if should_go_back {
                // Restore previous word's typed input
                test_state.typed_input = restored_input;

                // Restore previous word's error state by checking character results
                let has_error = if prev_idx < self.word_states.len() {
                    let word_state = &self.word_states[prev_idx];
                    word_state
                        .results
                        .iter()
                        .take(test_state.typed_input.len())
                        .any(|&result| result == CharResult::Incorrect)
                } else {
                    false
                };
                test_state.current_word_has_error = has_error;

                // Remove from words_with_errors since we're re-editing it
                self.words_with_errors.retain(|&idx| idx != prev_idx);

                // Move to previous word
                test_state.current_word_idx = prev_idx;
            } else {
                // Normal backspace on current word
                if current_word_idx < self.word_states.len() {
                    test_state.on_backspace(&mut self.word_states[current_word_idx]);
                }
            }
        }
    }

    /// Handles space being pressed.
    pub fn on_space(&mut self) {
        if let CurrentScreen::TypingTest(ref mut test_state) = &mut self.state {
            if test_state.current_word_idx >= self.word_states.len() {
                return;
            }

            let word_state = &self.word_states[test_state.current_word_idx];

            // If word is not fully typed, treat space as an incorrect character
            // Only treat space as incorrect if user has started typing
            if !test_state.typed_input.is_empty()
                && test_state.typed_input.len() < word_state.target.len()
            {
                self.on_char(' ');
                return;
            }
            // If typed_input is empty, space does nothing (doesn't start timer)
            if test_state.typed_input.is_empty() {
                return;
            }

            // Word is complete, save typed input before advancing
            // results already contains the final state and won't be modified for completed words
            let current_word_idx = test_state.current_word_idx;
            if current_word_idx < self.word_states.len() {
                self.word_states[current_word_idx].typed = test_state.typed_input.clone();
            }

            // Word is complete, try to advance
            let all_completed = test_state.on_space(
                &self.word_states[current_word_idx],
                &mut self.words_with_errors,
                &self.start_time,
                self.test_mode,
                self.word_states.len(),
            );

            if all_completed {
                // Freeze the timer when test completes in Words mode
                if self.test_mode == TestMode::Words {
                    self.end_time = Some(Instant::now());
                }
                self.state = CurrentScreen::TestResults;
            }
        }
    }

    /// Returns the selected time duration in seconds.
    pub fn time_seconds(&self) -> u32 {
        if let CurrentScreen::Menu(ref menu) = &self.state {
            menu.time_seconds()
        } else {
            TIME_OPTIONS[0] // Default fallback
        }
    }

    /// Returns the selected target word count.
    pub fn target_word_count(&self) -> u32 {
        if let CurrentScreen::Menu(ref menu) = &self.state {
            menu.target_word_count()
        } else {
            WORD_OPTIONS[1] // Default fallback
        }
    }

    /// Returns the elapsed time since the test started.
    /// If the test has ended (Words mode), returns the duration between start and end.
    pub fn time_elapsed(&self) -> Duration {
        if let Some(end) = self.end_time {
            if let Some(start) = self.start_time {
                return end.duration_since(start);
            }
        }
        self.start_time.map_or(Duration::ZERO, |s| s.elapsed())
    }

    /// Returns the remaining time in timed mode.
    pub fn time_remaining(&self) -> Duration {
        match self.start_time {
            Some(start) => self.duration.saturating_sub(start.elapsed()),
            None => self.duration,
        }
    }

    /// Calculates and returns test statistics.
    ///
    /// WPM is calculated as (correct_chars / 5) / minutes.
    /// Accuracy is (correct_chars / total_chars) * 100.
    pub fn calculate_stats(&self) -> Stats {
        Stats::calculate(
            self.correct_chars,
            self.total_chars_typed,
            self.words_with_errors.len(),
            self.start_time,
            self.end_time,
            self.duration,
            self.test_mode,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_words() -> Vec<String> {
        vec!["the".into(), "quick".into(), "brown".into()]
    }

    fn running_app() -> App {
        App::with_words(test_words())
    }

    #[test]
    fn test_initial_state_is_menu() {
        let app = App::new();
        match &app.state {
            CurrentScreen::Menu(menu) => {
                assert_eq!(menu.menu_field, MenuField::Mode);
                assert_eq!(menu.time_option_idx, 1);
                assert_eq!(menu.word_option_idx, 1);
            }
            _ => panic!("Expected Menu state"),
        }
        assert_eq!(app.test_mode, TestMode::Time);
    }

    #[test]
    fn test_menu_navigation() {
        let mut app = App::new();
        assert!(
            matches!(&app.state, CurrentScreen::Menu(_)),
            "Expected Menu state"
        );
        if let CurrentScreen::Menu(menu) = &app.state {
            assert_eq!(menu.menu_field, MenuField::Mode);
        }

        app.menu_down();
        assert!(
            matches!(&app.state, CurrentScreen::Menu(_)),
            "Expected Menu state"
        );
        if let CurrentScreen::Menu(menu) = &app.state {
            assert_eq!(menu.menu_field, MenuField::Value);
        }

        app.menu_down();
        assert!(
            matches!(&app.state, CurrentScreen::Menu(_)),
            "Expected Menu state"
        );
        if let CurrentScreen::Menu(menu) = &app.state {
            assert_eq!(menu.menu_field, MenuField::Start);
        }

        app.menu_down();
        assert!(
            matches!(&app.state, CurrentScreen::Menu(_)),
            "Expected Menu state"
        );
        if let CurrentScreen::Menu(menu) = &app.state {
            assert_eq!(menu.menu_field, MenuField::Start);
        }

        app.menu_up();
        assert!(
            matches!(&app.state, CurrentScreen::Menu(_)),
            "Expected Menu state"
        );
        if let CurrentScreen::Menu(menu) = &app.state {
            assert_eq!(menu.menu_field, MenuField::Value);
        }

        app.menu_up();
        assert!(
            matches!(&app.state, CurrentScreen::Menu(_)),
            "Expected Menu state"
        );
        if let CurrentScreen::Menu(menu) = &app.state {
            assert_eq!(menu.menu_field, MenuField::Mode);
        }

        app.menu_up();
        assert!(
            matches!(&app.state, CurrentScreen::Menu(_)),
            "Expected Menu state"
        );
        if let CurrentScreen::Menu(menu) = &app.state {
            assert_eq!(menu.menu_field, MenuField::Mode);
        }
    }

    #[test]
    fn test_menu_toggle_mode() {
        let mut app = App::new();
        assert_eq!(app.test_mode, TestMode::Time);

        app.menu_right();
        assert_eq!(app.test_mode, TestMode::Words);

        app.menu_left();
        assert_eq!(app.test_mode, TestMode::Time);
    }

    #[test]
    fn test_menu_value_selection_time() {
        let mut app = App::new();
        assert!(
            matches!(&app.state, CurrentScreen::Menu(_)),
            "Expected Menu state"
        );
        if let CurrentScreen::Menu(menu) = &mut app.state {
            menu.menu_field = MenuField::Value;
        }

        assert!(
            matches!(&app.state, CurrentScreen::Menu(_)),
            "Expected Menu state"
        );
        assert_eq!(app.time_seconds(), 60);

        app.menu_right();
        assert_eq!(app.time_seconds(), 90);

        app.menu_right();
        assert_eq!(app.time_seconds(), 120);

        app.menu_right();
        assert_eq!(app.time_seconds(), 120); // stays at max

        app.menu_left();
        assert_eq!(app.time_seconds(), 90);

        app.menu_left();
        assert_eq!(app.time_seconds(), 60);
    }

    #[test]
    fn test_menu_value_selection_words() {
        let mut app = App::new();
        app.test_mode = TestMode::Words;
        assert!(
            matches!(&app.state, CurrentScreen::Menu(_)),
            "Expected Menu state"
        );
        if let CurrentScreen::Menu(menu) = &mut app.state {
            menu.menu_field = MenuField::Value;
        }

        assert!(
            matches!(&app.state, CurrentScreen::Menu(_)),
            "Expected Menu state"
        );
        assert_eq!(app.target_word_count(), 25);

        app.menu_left();
        assert_eq!(app.target_word_count(), 10);

        app.menu_left();
        assert_eq!(app.target_word_count(), 10); // stays at min

        app.menu_right();
        assert_eq!(app.target_word_count(), 25);

        // Navigate to max
        for _ in 0..10 {
            app.menu_right();
        }
        assert_eq!(app.target_word_count(), 200);
    }

    #[test]
    fn test_menu_start_begins_test() {
        let mut app = App::new();
        app.start_test();

        match &app.state {
            CurrentScreen::TypingTest(_) => {}
            _ => panic!("Expected TypingTest state"),
        }
        assert!(!app.word_states.is_empty());
    }

    #[test]
    fn test_first_char_starts_timer() {
        let mut app = running_app();
        assert!(app.start_time.is_none());

        app.on_char('t');
        assert!(app.start_time.is_some());
    }

    #[test]
    fn test_correct_typing() {
        let mut app = running_app();
        assert!(
            matches!(&app.state, CurrentScreen::TypingTest(_)),
            "Expected TypingTest state"
        );

        app.on_char('t');
        app.on_char('h');
        app.on_char('e');

        assert!(
            matches!(&app.state, CurrentScreen::TypingTest(_)),
            "Expected TypingTest state"
        );
        if let CurrentScreen::TypingTest(test_state) = &app.state {
            assert_eq!(test_state.typed_input, "the");
            assert!(!test_state.current_word_has_error);
        }
        assert_eq!(app.total_chars_typed, 3);
        assert_eq!(app.correct_chars, 3);
        assert!(app.word_states[0]
            .results
            .iter()
            .all(|r| *r == CharResult::Correct));
    }

    #[test]
    fn test_incorrect_typing() {
        let mut app = running_app();
        assert!(
            matches!(&app.state, CurrentScreen::TypingTest(_)),
            "Expected TypingTest state"
        );

        app.on_char('t');
        app.on_char('h');
        app.on_char('x');

        assert!(
            matches!(&app.state, CurrentScreen::TypingTest(_)),
            "Expected TypingTest state"
        );
        if let CurrentScreen::TypingTest(test_state) = &app.state {
            assert_eq!(test_state.typed_input, "thx");
            assert!(test_state.current_word_has_error);
        }
        assert_eq!(app.correct_chars, 2);
        assert_eq!(app.word_states[0].results[2], CharResult::Incorrect);
    }

    #[test]
    fn test_space_advances_word() {
        let mut app = running_app();
        assert!(
            matches!(&app.state, CurrentScreen::TypingTest(_)),
            "Expected TypingTest state"
        );

        for c in "the".chars() {
            app.on_char(c);
        }
        app.on_space();

        assert!(
            matches!(&app.state, CurrentScreen::TypingTest(_)),
            "Expected TypingTest state"
        );
        if let CurrentScreen::TypingTest(test_state) = &app.state {
            assert_eq!(test_state.current_word_idx, 1);
            assert!(test_state.typed_input.is_empty());
            assert!(!test_state.current_word_has_error);
        }
    }

    #[test]
    fn test_space_does_not_advance_before_typing() {
        let mut app = running_app();
        assert!(
            matches!(&app.state, CurrentScreen::TypingTest(_)),
            "Expected TypingTest state"
        );
        app.on_space();

        assert!(
            matches!(&app.state, CurrentScreen::TypingTest(_)),
            "Expected TypingTest state"
        );
        if let CurrentScreen::TypingTest(test_state) = &app.state {
            assert_eq!(test_state.current_word_idx, 0);
        }
        assert!(app.start_time.is_none());
    }

    #[test]
    fn test_backspace() {
        let mut app = running_app();
        assert!(
            matches!(&app.state, CurrentScreen::TypingTest(_)),
            "Expected TypingTest state"
        );

        for c in "thx".chars() {
            app.on_char(c);
        }
        assert_eq!(app.word_states[0].results.len(), 3);

        app.on_backspace();

        assert!(
            matches!(&app.state, CurrentScreen::TypingTest(_)),
            "Expected TypingTest state"
        );
        if let CurrentScreen::TypingTest(test_state) = &app.state {
            assert_eq!(test_state.typed_input, "th");
            assert_eq!(
                app.word_states[test_state.current_word_idx].results.len(),
                3
            );
        }
    }

    #[test]
    fn test_backspace_on_empty_does_nothing() {
        let mut app = running_app();
        assert!(
            matches!(&app.state, CurrentScreen::TypingTest(_)),
            "Expected TypingTest state"
        );
        app.on_char('t');
        app.on_backspace();
        app.on_backspace();

        assert!(
            matches!(&app.state, CurrentScreen::TypingTest(_)),
            "Expected TypingTest state"
        );
        if let CurrentScreen::TypingTest(test_state) = &app.state {
            assert!(test_state.typed_input.is_empty());
        }
    }

    #[test]
    fn test_time_remaining_before_start() {
        let app = running_app();
        assert_eq!(app.time_remaining(), Duration::from_secs(60));
    }

    #[test]
    fn test_time_remaining_after_start() {
        let mut app = running_app();
        app.on_char('t');

        assert!(app.time_remaining() <= Duration::from_secs(60));
        assert!(app.time_remaining() > Duration::from_secs(59));
    }

    #[test]
    fn test_stats_accuracy_all_correct() {
        let mut app = running_app();

        for c in "the".chars() {
            app.on_char(c);
        }

        let stats = app.calculate_stats();
        assert_eq!(stats.accuracy, 100.0);
        assert_eq!(stats.total_errors, 0);
    }

    #[test]
    fn test_stats_accuracy_with_errors() {
        let mut app = running_app();

        for c in "thxe".chars() {
            app.on_char(c);
        }

        let stats = app.calculate_stats();
        assert_eq!(stats.correct_chars, 2);
        assert_eq!(stats.total_chars_typed, 4);
        assert_eq!(stats.accuracy, 50.0);
        assert_eq!(stats.total_errors, 2);
    }

    #[test]
    fn test_stats_words_with_errors() {
        let mut app = running_app();

        for c in "the".chars() {
            app.on_char(c);
        }
        app.on_space();

        for c in "qxick".chars() {
            app.on_char(c);
        }
        app.on_space();

        let stats = app.calculate_stats();
        assert_eq!(stats.words_with_errors, 1);
    }

    #[test]
    fn test_reset_returns_to_menu() {
        let mut app = running_app();
        assert!(
            matches!(&app.state, CurrentScreen::TypingTest(_)),
            "Expected TypingTest state"
        );

        for c in "the".chars() {
            app.on_char(c);
        }
        app.on_space();
        app.reset();

        assert!(
            matches!(&app.state, CurrentScreen::Menu(_)),
            "Expected Menu state"
        );
        match &app.state {
            CurrentScreen::Menu(_) => {}
            _ => panic!("Expected Menu state"),
        }
        assert_eq!(app.total_chars_typed, 0);
        assert!(app.start_time.is_none());
        assert!(app.words_with_errors.is_empty());
    }

    #[test]
    fn test_typing_after_finished_does_nothing() {
        let mut app = running_app();
        app.state = CurrentScreen::TestResults;

        app.on_char('t');

        match &app.state {
            CurrentScreen::TestResults => {}
            _ => panic!("Expected TestResults state"),
        }
        assert_eq!(app.total_chars_typed, 0);
    }

    #[test]
    fn test_space_after_finished_does_nothing() {
        let mut app = running_app();
        app.on_char('t');
        app.state = CurrentScreen::TestResults;
        app.on_space();

        match &app.state {
            CurrentScreen::TestResults => {}
            _ => panic!("Expected TestResults state"),
        }
    }

    #[test]
    fn test_multiple_words_typed() {
        let mut app = running_app();
        assert!(
            matches!(&app.state, CurrentScreen::TypingTest(_)),
            "Expected TypingTest state"
        );

        for c in "the".chars() {
            app.on_char(c);
        }
        app.on_space();

        for c in "quick".chars() {
            app.on_char(c);
        }
        app.on_space();

        assert!(
            matches!(&app.state, CurrentScreen::TypingTest(_)),
            "Expected TypingTest state"
        );
        if let CurrentScreen::TypingTest(test_state) = &app.state {
            assert_eq!(test_state.current_word_idx, 2);
        }
        assert_eq!(app.correct_chars, 8);
    }

    #[test]
    fn test_words_mode_finishes_on_last_word() {
        let mut app = App::with_words(test_words());
        app.test_mode = TestMode::Words;

        for word in ["the", "quick", "brown"] {
            for c in word.chars() {
                app.on_char(c);
            }
            app.on_space();
        }

        assert_eq!(app.state, CurrentScreen::TestResults);
    }

    #[test]
    fn test_words_mode_finishes_on_last_letter_without_space() {
        let mut app = App::with_words(test_words());
        app.test_mode = TestMode::Words;

        // Type first two words with spaces
        for word in ["the", "quick"] {
            for c in word.chars() {
                app.on_char(c);
            }
            app.on_space();
        }

        // Type the last word - test should end when last letter is typed
        for c in "brown".chars() {
            app.on_char(c);
        }

        // Test should have ended without needing a space
        assert_eq!(app.state, CurrentScreen::TestResults);
        assert!(app.end_time.is_some());
    }

    #[test]
    fn test_time_mode_does_not_finish_on_last_word() {
        let mut app = App::with_words(test_words());
        app.test_mode = TestMode::Time;

        for word in ["the", "quick", "brown"] {
            for c in word.chars() {
                app.on_char(c);
            }
            app.on_space();
        }

        match &app.state {
            CurrentScreen::TypingTest(_) => {}
            _ => panic!("Expected TypingTest state"),
        }
    }
}
