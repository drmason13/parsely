//! The error returned when a parser or lexer does not match successfully.
//!
//! Parsely's error handling strategy is currently unstable. Expect these types to change.
//!
//! ## The approach to error handling:
//!
//! In line with parsely's aims, error handling is simple.
//!
//! If a parser or lexer finds that the input does not match, it returns an error with the following information:
//!
//! * where in the input (that it can see) the match failed
//! * what it expected to see
//!
//! If a parser or lexer runs a lexer or parser which returns an error it has a choice:
//!
//! * fail and return the error
//! * continue without failing
//!
//! Every time a lexer is running and making these decisions regarding errors, it is only privy to the part of the input that it has been given.

use std::{borrow::Cow, cmp::min, fmt};

/// This is a simple "all the possible errors while parsing" enum.
///
/// In the future this will contain some intermediate state of the parsing/lexing to aid in debugging.
#[non_exhaustive]
#[derive(PartialEq, Debug)]
pub enum Error {
    /// A lexer did not see the expected input and has failed to match
    NoMatch,

    /// When converting to the output type there was an error
    FailedConversion,
}

impl std::error::Error for Error {}
impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::NoMatch => write!(f, "No Match"),
            Error::FailedConversion => write!(f, "Failed to convert matched input"),
        }
    }
}

pub struct UnexpectedInput<'i> {
    expected: Expected,
    found: Found<'i>,
}

pub enum Expected {
    /// The fixed literal str input that this lexer matches:
    ///
    /// For example:
    /// * `char('a')` expects "a"
    /// * `token("foo")` expects "foo"
    ///
    /// If there are multiple possible inputs the lexer can match (as is often the case!) then see [`Expected::Labelled`]
    Str(&'static str),

    /// A single char that is expected
    Char(char),

    /// The
    Labelled {
        /// A description of the possible inputs this lexer matches.
        ///
        /// # Examples
        ///
        /// * [`uppercase`](crate::uppercase) uses the static label `uppercase letter`
        /// * [`digit`](crate::digit) uses the static label `digit`
        /// * `uppercase().many(1..=3)` uses the label `1 to 3 uppercase letter` built dynamically from the label of uppercase and its range.
        /// * `uppercase().many(1..=3).or(digit())` uses the label "(1 to 3 uppercase letter or digit)" joining the label of its left and right items with "(... or ...)".
        /// * `uppercase().or(digit()).many(1..=3)` uses the label "1 to 3 (uppercase letter or digit)".
        /// * `uppercase().or(digit()).or(token("foo")` uses the label "(uppercase letter or (digit or `foo`))".
        ///
        /// You can set a label for your own parsers by using `.label("my label")` which can help shorten overly long chains of combinators to make error messages easier to understand.
        label: Cow<'static, str>,

        /// How many chars worth of input to show was found when displaying the error.
        width: usize,
    },

    EndOfInput,
}

pub struct Found<'i> {
    content: &'i str,
    offset: usize,
}

impl<'i> UnexpectedInput<'i> {
    pub fn new(expected: Expected, input: &'i str, offset: usize) -> UnexpectedInput<'i> {
        let start = offset;
        let width = match expected {
            Expected::Str(s) => s.len(),
            Expected::Char(_) => 1,
            Expected::Labelled { label: _, width } => width,
            Expected::EndOfInput => 1,
        };

        let start = min(offset, input.len());
        let end = min(offset + width, input.len());

        let found = Found {
            content: &input[start..end],
            offset,
        };

        let expected = todo!();

        UnexpectedInput { expected, found }
    }
}

#[cfg(test)]
mod error_tests {
    use super::*;
    use crate::{char, int, Lex, Parse};

    #[test]
    fn error_message() {
        let parser = int::<u8>().many(1..=3).then_skip(char('!'));

        let error_message = match parser.parse("123") {
            Err(e) => format!("{e}"),
            Ok(_) => panic!("should error"),
        };

        assert_eq!(
            error_message,
            "expected [123456789] found `.` at line 1 char 0"
        );

        let error_message = match parser.parse("...") {
            Err(e) => format!("{e}"),
            Ok(_) => panic!("should error"),
        };

        assert_eq!(
            error_message,
            "expected `!` found <end of input> at line 1 char 3"
        );

        let error_message = match parser.parse("123") {
            Err(e) => format!("{e}"),
            Ok(_) => panic!("should error"),
        };

        assert_eq!(
            error_message,
            "expected `!` found <end of input> at line 1 char 3"
        );
    }
}
