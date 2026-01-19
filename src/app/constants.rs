/// Available time options for timed mode (in seconds).
pub const TIME_OPTIONS: &[u32] = &[30, 60, 90, 120];

/// Available word count options for words mode.
pub const WORD_OPTIONS: &[u32] = &[10, 25, 50, 100, 150, 200];

/// Punctuation probability constants (0.0 - 1.0).
pub mod punctuation {
    /// Chance to end a word with a period (triggers capitalization of next word).
    pub const PERIOD_CHANCE: f64 = 0.10;
    /// Chance to end a word with a question mark.
    pub const QUESTION_CHANCE: f64 = 0.01;
    /// Chance to end a word with an exclamation mark.
    pub const EXCLAMATION_CHANCE: f64 = 0.01;
    /// Chance to add a comma after a word.
    pub const COMMA_CHANCE: f64 = 0.15;
    /// Chance to add a semicolon after a word.
    pub const SEMICOLON_CHANCE: f64 = 0.015;
    /// Chance to add a colon after a word.
    pub const COLON_CHANCE: f64 = 0.013;
    /// Chance to wrap word in double quotes.
    pub const QUOTE_CHANCE: f64 = 0.01;
    /// Chance to wrap word in parentheses.
    pub const PAREN_CHANCE: f64 = 0.012;
}
