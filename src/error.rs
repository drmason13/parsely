//! The error returned when a parser or lexer does not match successfully.
//!
//! Parsely's error handling strategy is currently unstable. Expect these types to change.

use std::fmt;

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
