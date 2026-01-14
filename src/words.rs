//! Word generation for the typing test.
//!
//! This module provides a pool of common English words and a function
//! to generate random word lists for typing tests.

use rand::prelude::IndexedRandom;
use rand::rng;
use serde::Deserialize;
use std::sync::OnceLock;

/// Structure for the words JSON file.
#[derive(Debug, Deserialize)]
struct WordSet {
    #[allow(unused)]
    name: String,
    #[allow(unused)]
    #[serde(rename = "orderedByFrequency")]
    ordered_by_frequency: bool,
    words: Vec<String>,
}

/// Pool of common English words used in typing tests.
///
/// Words are loaded from the words.json file at compile time.
/// Words are selected from the most common English words to ensure
/// familiarity and reasonable typing difficulty.
fn get_words() -> &'static [String] {
    static WORDS: OnceLock<Vec<String>> = OnceLock::new();
    WORDS.get_or_init(|| {
        let words_json = include_str!("../assets/words.json");
        let words_data: WordSet = serde_json::from_str(words_json)
            .expect("Failed to parse assets/words.json");
        words_data.words
    }).as_slice()
}

/// Generates a random list of words for a typing test.
///
/// Words are randomly selected from the word pool with replacement,
/// meaning the same word may appear multiple times in the list.
///
/// # Arguments
///
/// * `count` - The number of words to generate
///
/// # Returns
///
/// A vector of randomly selected words
///
/// # Example
///
/// ```
/// let words = generate_words(25);
/// assert_eq!(words.len(), 25);
/// ```
pub fn generate_words(count: usize) -> Vec<String> {
    let mut rng = rng();
    let words_pool = get_words();
    let mut words: Vec<String> = Vec::with_capacity(count);

    for _ in 0..count {
        if let Some(word) = words_pool.choose(&mut rng) {
            words.push(word.clone());
        }
    }

    words
}
