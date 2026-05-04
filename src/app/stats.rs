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

    /// Elapsed test time in seconds (same basis as WPM in [`Self::calculate`]).
    pub fn test_elapsed_secs(
        start_time: Option<Instant>,
        end_time: Option<Instant>,
        duration: Duration,
        test_mode: TestMode,
    ) -> f64 {
        Self::calculate_elapsed_secs(start_time, end_time, duration, test_mode)
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    fn make_start_time() -> Instant {
        Instant::now()
    }

    fn make_end_time(start: Instant, secs: f64) -> Instant {
        start + Duration::from_secs_f64(secs)
    }

    #[test]
    fn test_wpm_calculation_basic() {
        let start = make_start_time();
        let end = make_end_time(start, 60.0);

        // 300 correct chars = 60 words in 1 minute = 60 WPM
        let stats = Stats::calculate(
            300,
            300,
            0,
            Some(start),
            Some(end),
            Duration::from_secs(60),
            TestMode::Words,
        );

        assert!(
            (stats.wpm - 60.0).abs() < 0.01,
            "Expected ~60 WPM, got {}",
            stats.wpm
        );
    }

    #[test]
    fn test_wpm_calculation_30_seconds() {
        let start = make_start_time();
        let end = make_end_time(start, 30.0); // 30 seconds = 0.5 minutes

        // 150 correct chars = 30 words in 0.5 minutes = 60 WPM
        let stats = Stats::calculate(
            150,
            150,
            0,
            Some(start),
            Some(end),
            Duration::from_secs(60),
            TestMode::Words,
        );

        assert!(
            (stats.wpm - 60.0).abs() < 0.01,
            "Expected ~60 WPM, got {}",
            stats.wpm
        );
    }

    #[test]
    fn test_wpm_zero_time() {
        let start = make_start_time();

        // No time elapsed should result in 0 WPM
        let stats = Stats::calculate(
            100,
            100,
            0,
            Some(start),
            Some(start), // Same as start = 0 elapsed
            Duration::from_secs(60),
            TestMode::Words,
        );

        assert_eq!(stats.wpm, 0.0);
    }

    #[test]
    fn test_wpm_no_start_time() {
        // No start time should result in 0 WPM
        let stats = Stats::calculate(
            100,
            100,
            0,
            None,
            None,
            Duration::from_secs(60),
            TestMode::Words,
        );

        assert_eq!(stats.wpm, 0.0);
    }

    #[test]
    fn test_accuracy_all_correct() {
        let start = make_start_time();
        let end = make_end_time(start, 60.0);

        let stats = Stats::calculate(
            100,
            100,
            0,
            Some(start),
            Some(end),
            Duration::from_secs(60),
            TestMode::Words,
        );

        assert_eq!(stats.accuracy, 100.0);
        assert_eq!(stats.total_errors, 0);
    }

    #[test]
    fn test_accuracy_all_incorrect() {
        let start = make_start_time();
        let end = make_end_time(start, 60.0);

        let stats = Stats::calculate(
            0,
            100,
            0,
            Some(start),
            Some(end),
            Duration::from_secs(60),
            TestMode::Words,
        );

        assert_eq!(stats.accuracy, 0.0);
        assert_eq!(stats.total_errors, 100);
    }

    #[test]
    fn test_accuracy_partial() {
        let start = make_start_time();
        let end = make_end_time(start, 60.0);

        // 75 correct out of 100 = 75% accuracy
        let stats = Stats::calculate(
            75,
            100,
            0,
            Some(start),
            Some(end),
            Duration::from_secs(60),
            TestMode::Words,
        );

        assert_eq!(stats.accuracy, 75.0);
        assert_eq!(stats.total_errors, 25);
    }

    #[test]
    fn test_accuracy_zero_chars() {
        let start = make_start_time();
        let end = make_end_time(start, 60.0);

        // Zero characters typed should return 100% accuracy
        let stats = Stats::calculate(
            0,
            0,
            0,
            Some(start),
            Some(end),
            Duration::from_secs(60),
            TestMode::Words,
        );

        assert_eq!(stats.accuracy, 100.0);
        assert_eq!(stats.total_errors, 0);
    }

    #[test]
    fn test_fractional_accuracy() {
        let start = make_start_time();
        let end = make_end_time(start, 60.0);

        // 1 correct out of 3 = 33.333...%
        let stats = Stats::calculate(
            1,
            3,
            0,
            Some(start),
            Some(end),
            Duration::from_secs(60),
            TestMode::Words,
        );

        assert!((stats.accuracy - 33.333).abs() < 0.1);
        assert_eq!(stats.total_errors, 2);
    }
}
