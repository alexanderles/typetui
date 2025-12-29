//! Typing test state and input handling logic.

use super::types::{CharResult, TestMode, WordState};
use std::time::Instant;

/// State for an active typing test.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TypingTestState {
    /// Index of the word currently being typed.
    pub current_word_idx: usize,
    /// Characters typed for the current word.
    pub typed_input: String,
    /// Whether the current word has any errors.
    pub current_word_has_error: bool,
}

impl TypingTestState {
    /// Creates a new TypingTestState.
    pub fn new() -> Self {
        Self {
            current_word_idx: 0,
            typed_input: String::new(),
            current_word_has_error: false,
        }
    }

    /// Handles a character being typed.
    ///
    /// Returns whether the character was correct.
    pub fn on_char(
        &mut self,
        c: char,
        word_state: &mut WordState,
        total_chars_typed: &mut usize,
        correct_chars: &mut usize,
        start_time: &mut Option<Instant>,
    ) -> bool {
        // Start timer on first character
        if start_time.is_none() {
            *start_time = Some(Instant::now());
        }

        let char_pos = self.typed_input.len();
        let expected_char = word_state.target.chars().nth(char_pos);
        let is_correct = expected_char == Some(c);

        *total_chars_typed += 1;

        if is_correct {
            *correct_chars += 1;
            // Check if this position was previously Incorrect or Corrected
            // The results array preserves full history even after backspacing
            let previous_result = word_state.results.get(char_pos).copied();
            let was_incorrect = previous_result == Some(CharResult::Incorrect);
            let was_corrected = previous_result == Some(CharResult::Corrected);

            // If we're correcting a previously incorrect character, or retyping a Corrected position, mark it as Corrected
            if was_incorrect || was_corrected {
                // Ensure results array is long enough, then set the result
                while word_state.results.len() <= char_pos {
                    word_state.results.push(CharResult::Correct);
                }
                word_state.results[char_pos] = CharResult::Corrected;
            } else {
                // Ensure results array is long enough, then set the result
                while word_state.results.len() <= char_pos {
                    word_state.results.push(CharResult::Correct);
                }
                word_state.results[char_pos] = CharResult::Correct;
            }
        } else {
            self.current_word_has_error = true;
            // Ensure results array is long enough, then set the result
            while word_state.results.len() <= char_pos {
                word_state.results.push(CharResult::Correct);
            }
            word_state.results[char_pos] = CharResult::Incorrect;
        }

        self.typed_input.push(c);
        is_correct
    }

    /// Handles backspace being pressed.
    ///
    /// Removes the last typed character but preserves the full results history.
    /// The results array is not truncated, allowing correction state to persist
    /// even when backspacing across word boundaries.
    pub fn on_backspace(&mut self, _word_state: &mut WordState) {
        if self.typed_input.is_empty() {
            return;
        }

        self.typed_input.pop();
    }

    /// Handles space being pressed.
    ///
    /// If the word is complete, advances to the next word.
    /// Returns true if all words are completed (for Words mode).
    pub fn on_space(
        &mut self,
        word_state: &WordState,
        words_with_errors: &mut Vec<usize>,
        start_time: &Option<Instant>,
        test_mode: TestMode,
        total_words: usize,
    ) -> bool {
        if start_time.is_none() {
            return false;
        }

        if self.current_word_idx >= total_words {
            return false;
        }

        // Word must be complete (or over-typed) to advance
        // Incomplete words are handled in App::on_space by treating space as an error
        if self.typed_input.len() < word_state.target.len() {
            return false;
        }

        // Word is complete (or over-typed), advance to next word
        let has_error =
            self.current_word_has_error || self.typed_input.len() > word_state.target.len();
        if has_error {
            words_with_errors.push(self.current_word_idx);
        }

        self.current_word_idx += 1;
        self.typed_input.clear();
        self.current_word_has_error = false;

        // Return true if all words completed (for Words mode)
        test_mode == TestMode::Words && self.current_word_idx >= total_words
    }
}

impl Default for TypingTestState {
    fn default() -> Self {
        Self::new()
    }
}
