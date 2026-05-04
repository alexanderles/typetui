//! Cumulative WPM series for the results chart.

/// For each whole second `s` from 1 through `floor(elapsed_secs)`, WPM is
/// `(correct_chars through time s) / 5 / (s / 60)`.
pub fn build_cumulative_wpm_points(
    samples: &[(f64, usize)],
    elapsed_secs: f64,
) -> Vec<(f64, f64)> {
    if samples.is_empty() || elapsed_secs <= 0.0 {
        return Vec::new();
    }
    let last_sec = elapsed_secs.floor() as u32;
    if last_sec == 0 {
        return Vec::new();
    }
    let mut points = Vec::with_capacity(last_sec as usize);
    for s in 1..=last_sec {
        let correct = correct_chars_at_or_before(samples, s as f64);
        let wpm = (correct as f64 / 5.0) / (s as f64 / 60.0);
        points.push((s as f64, wpm));
    }
    points
}

fn correct_chars_at_or_before(samples: &[(f64, usize)], deadline_secs: f64) -> usize {
    samples
        .iter()
        .filter(|(t, _)| *t <= deadline_secs)
        .last()
        .map(|(_, c)| *c)
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_samples_yields_empty() {
        assert!(build_cumulative_wpm_points(&[], 10.0).is_empty());
    }

    #[test]
    fn zero_or_negative_elapsed_yields_empty() {
        assert!(build_cumulative_wpm_points(&[(0.5, 5)], 0.0).is_empty());
        assert!(build_cumulative_wpm_points(&[(0.5, 5)], -1.0).is_empty());
    }

    #[test]
    fn subsecond_elapsed_yields_empty() {
        assert!(build_cumulative_wpm_points(&[(0.1, 3)], 0.9).is_empty());
    }

    #[test]
    fn single_second_uses_samples_through_one() {
        let samples = vec![(0.2, 10), (0.8, 30)];
        let pts = build_cumulative_wpm_points(&samples, 1.2);
        assert_eq!(pts.len(), 1);
        assert!((pts[0].0 - 1.0).abs() < f64::EPSILON);
        assert!((pts[0].1 - 360.0).abs() < 0.01);
    }

    #[test]
    fn later_seconds_only_count_samples_up_to_deadline() {
        let samples = vec![(0.5, 5), (1.5, 15), (2.2, 40)];
        let pts = build_cumulative_wpm_points(&samples, 3.0);
        assert_eq!(pts.len(), 3);
        assert!((pts[0].1 - 60.0).abs() < 0.01);
        assert!((pts[1].1 - 90.0).abs() < 0.01);
        assert!((pts[2].1 - 160.0).abs() < 0.01);
    }

    #[test]
    fn decreasing_correct_chars_uses_last_sample_not_max() {
        let samples = vec![(0.5, 10), (1.5, 20), (2.0, 5)];
        let pts = build_cumulative_wpm_points(&samples, 3.0);
        assert_eq!(pts.len(), 3);
        let wpm2 = (5.0 / 5.0) / (2.0 / 60.0);
        assert!((pts[1].1 - wpm2).abs() < 0.01);
    }
}
