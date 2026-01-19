//! Shared types and enums for the typing test application.

/// The type of typing test.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TestMode {
    /// Test runs for a fixed duration.
    Time,
    /// Test runs until a fixed number of words are typed.
    Words,
}

/// The currently selected field in the menu.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MenuField {
    /// Mode selection (Time/Words).
    Mode,
    /// Value selection (duration or word count).
    Value,
    /// Punctuation toggle (On/Off).
    Punctuation,
    /// Start button.
    Start,
}

/// Result of typing a single character.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CharResult {
    /// Character matched the expected character.
    Correct,
    /// Character did not match the expected character.
    Incorrect,
    /// Character was initially incorrect but then corrected.
    Corrected,
}

/// Complete state for a single word in the typing test.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct WordState {
    /// The target word to type.
    pub target: String,
    /// What was actually typed for this word.
    pub typed: String,
    /// Character-by-character correctness results.
    pub results: Vec<CharResult>,
}

/// The current state of the application.
///
/// Note: MenuState and TypingTestState are defined in their respective modules
/// to avoid circular dependencies. This enum uses them via type aliases.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CurrentScreen {
    /// User is on the menu screen selecting test options.
    Menu(crate::app::menu::MenuState),
    /// User is actively taking the typing test.
    TypingTest(crate::app::typing_test::TypingTestState),
    /// Test is complete and results are being displayed.
    TestResults,
}
