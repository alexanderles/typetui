//! Word generation for the typing test.
//!
//! This module provides a pool of common English words and a function
//! to generate random word lists for typing tests.

use rand::prelude::IndexedRandom;
use rand::{rng, Rng};
use serde::Deserialize;
use std::sync::OnceLock;

use crate::app::constants::punctuation::{
    COLON_CHANCE, COMMA_CHANCE, EXCLAMATION_CHANCE, PAREN_CHANCE, PERIOD_CHANCE, QUESTION_CHANCE,
    QUOTE_CHANCE, SEMICOLON_CHANCE,
};

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
/// * `punctuation` - Whether to add punctuation to the generated words
///
/// # Returns
///
/// A vector of randomly selected words
///
/// # Example
///
/// ```
/// let words = generate_words(25, false);
/// assert_eq!(words.len(), 25);
/// ```
pub fn generate_words(count: usize, punctuation: bool) -> Vec<String> {
    let mut rng = rng();
    let words_pool = get_words();
    let mut words: Vec<String> = Vec::with_capacity(count);

    for _ in 0..count {
        if let Some(word) = words_pool.choose(&mut rng) {
            words.push(word.clone());
        }
    }

    if punctuation {
        apply_punctuation(&mut words);
    }

    words
}

/// Capitalizes the first letter of a word.
fn capitalize_first(word: &str) -> String {
    let mut chars = word.chars();
    match chars.next() {
        None => String::new(),
        Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
    }
}

/// Checks if a character is sentence-ending punctuation.
fn is_sentence_ending(c: char) -> bool {
    matches!(c, '.' | '?' | '!')
}

/// Applies punctuation to a list of words.
///
/// Modifies words in place to add punctuation based on
/// probability constants.
/// - Capitalizes the first word
/// - Randomly adds punctuation after words
/// - Capitalizes words after sentence-ending punctuation
/// - Wraps some words in quotes or parentheses
/// - Ensures the last word ends with a period
fn apply_punctuation(words: &mut Vec<String>) {
    if words.is_empty() {
        return;
    }

    let mut rng = rng();
    let mut capitalize_next = true;

    for i in 0..words.len() {
        let is_last = i == words.len() - 1;

        // Capitalize if needed (first word or after sentence-ending punctuation)
        if capitalize_next {
            words[i] = capitalize_first(&words[i]);
            capitalize_next = false;
        }

        // Check previous word for existing punctuation (avoid double punctuation)
        let prev_has_punct = if i > 0 {
            words[i - 1]
                .chars()
                .last()
                .is_some_and(|c| matches!(c, '.' | ',' | '?' | '!' | ';' | ':' | '"' | ')'))
        } else {
            false
        };

        if is_last {
            // Always end last word with a period (unless already has sentence-ending punct)
            let last_char = words[i].chars().last();
            if !last_char.is_some_and(is_sentence_ending) {
                words[i].push('.');
            }
        } else if !prev_has_punct {
            // Roll for punctuation (mutually exclusive checks)
            let roll: f64 = rng.random();

            if roll < QUOTE_CHANCE {
                // Wrap in quotes
                words[i] = format!("\"{}\"", words[i]);
            } else if roll < QUOTE_CHANCE + PAREN_CHANCE {
                // Wrap in parentheses
                words[i] = format!("({})", words[i]);
            } else if roll < QUOTE_CHANCE + PAREN_CHANCE + PERIOD_CHANCE {
                // Add period
                words[i].push('.');
                capitalize_next = true;
            } else if roll < QUOTE_CHANCE + PAREN_CHANCE + PERIOD_CHANCE + QUESTION_CHANCE {
                // Add question mark
                words[i].push('?');
                capitalize_next = true;
            } else if roll
                < QUOTE_CHANCE + PAREN_CHANCE + PERIOD_CHANCE + QUESTION_CHANCE + EXCLAMATION_CHANCE
            {
                // Add exclamation mark
                words[i].push('!');
                capitalize_next = true;
            } else if roll
                < QUOTE_CHANCE
                    + PAREN_CHANCE
                    + PERIOD_CHANCE
                    + QUESTION_CHANCE
                    + EXCLAMATION_CHANCE
                    + COMMA_CHANCE
            {
                // Add comma
                words[i].push(',');
            } else if roll
                < QUOTE_CHANCE
                    + PAREN_CHANCE
                    + PERIOD_CHANCE
                    + QUESTION_CHANCE
                    + EXCLAMATION_CHANCE
                    + COMMA_CHANCE
                    + SEMICOLON_CHANCE
            {
                // Add semicolon
                words[i].push(';');
            } else if roll
                < QUOTE_CHANCE
                    + PAREN_CHANCE
                    + PERIOD_CHANCE
                    + QUESTION_CHANCE
                    + EXCLAMATION_CHANCE
                    + COMMA_CHANCE
                    + SEMICOLON_CHANCE
                    + COLON_CHANCE
            {
                // Add colon
                words[i].push(':');
            }
        }
    }
}
