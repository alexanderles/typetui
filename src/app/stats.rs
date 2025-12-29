//! Statistics calculation for typing tests.

use super::types::TestMode;
use std::time::{Duration, Instant};

/// Statistics calculated at the end of a typing test.
#[derive(Debug)]
pub struct Stats {
    /// Words per minute (based on 5 characters = 1 word).
    pub wpm: f64,
    /// Percentage of correctly typed characters.
    pub accuracy: f64,
    /// Total number of incorrectly typed characters.
    pub total_errors: usize,
    /// Number of words that contained at least one error.
    pub words_with_errors: usize,
    /// Total characters typed (correct + incorrect).
    pub total_chars_typed: usize,
    /// Number of correctly typed characters.
    pub correct_chars: usize,
}

impl Stats {
    /// Calculates and returns test statistics.
    ///
    /// WPM is calculated as (correct_chars / 5) / minutes.
    /// Accuracy is (correct_chars / total_chars) * 100.
    pub fn calculate(
        correct_chars: usize,
        total_chars_typed: usize,
        words_with_errors: usize,
        start_time: Option<Instant>,
        end_time: Option<Instant>,
        duration: Duration,
        test_mode: TestMode,
    ) -> Self {
        let elapsed_secs = Self::calculate_elapsed_secs(start_time, end_time, duration, test_mode);
        let elapsed_mins = elapsed_secs / 60.0;

        let wpm = if elapsed_mins > 0.0 {
            (correct_chars as f64 / 5.0) / elapsed_mins
        } else {
            0.0
        };

        let accuracy = if total_chars_typed > 0 {
            (correct_chars as f64 / total_chars_typed as f64) * 100.0
        } else {
            100.0
        };

        Stats {
            wpm,
            accuracy,
            total_errors: total_chars_typed - correct_chars,
            words_with_errors,
            total_chars_typed,
            correct_chars,
        }
    }

    /// Calculates elapsed seconds, capped at duration for timed mode.
    fn calculate_elapsed_secs(
        start_time: Option<Instant>,
        end_time: Option<Instant>,
        duration: Duration,
        test_mode: TestMode,
    ) -> f64 {
        match start_time {
            Some(start) => {
                let elapsed = if let Some(end) = end_time {
                    // Use frozen time if test has ended
                    end.duration_since(start)
                } else {
                    // Use current elapsed time if test is still running
                    start.elapsed()
                };
                match test_mode {
                    TestMode::Time => elapsed.min(duration).as_secs_f64(),
                    TestMode::Words => elapsed.as_secs_f64(),
                }
            }
            None => 0.0,
        }
    }
}
