//! The error returned when a parser or lexer does not match successfully.
//!
//! Parsely's error handling strategy is currently unstable. Expect these types to change.

use std::fmt;

/// The Error type returned by both [`parse`](crate::Parse::parse()) and [`lex`](crate::Lex::lex()) methods.
///
/// Errors in parsely don't directly capture a Span like most parsing libraries.
///
/// They simply store two slices of the original &str input: `remainder` and `input`
#[derive(PartialEq, Debug)]
pub struct Error<'i> {
    /// The reason for the error
    pub reason: ErrorReason,

    /// The remaining unparsed input
    pub remainder: &'i str,

    /// The input to the first parser to run, the *original* input
    pub input: &'i str,
}

impl<'i> Error<'i> {
    /// Create a new error at the point that a lexer failed to match the input
    ///
    /// See [`ErrorReason::NoMatch`]
    pub fn no_match(input: &'i str) -> Self {
        Error {
            input,
            remainder: input,
            reason: ErrorReason::NoMatch,
        }
    }

    /// Create a new error at the point that a parser failed to convert matched input into the output type
    ///
    /// See [`ErrorReason::FailedConversion`]
    pub fn failed_conversion(input: &'i str) -> Self {
        Error {
            input,
            remainder: input,
            reason: ErrorReason::FailedConversion,
        }
    }

    /// Update an existing error with the most recently seen input
    ///
    /// This is the mechanism by which we eventually find the original input (`error.input`) that the entire parser chain first saw.
    pub fn offset(mut self, input: &'i str) -> Self {
        self.input = input;
        self
    }

    /// Returns the part of the input that was matched overall before failure.
    ///
    /// Warning: This is derived from the current state of the error, so it can't be relied upon to be accurate *during* parsing.
    ///
    /// Note: this is exactly from the start of the input up until the point where the parser failed to match (`error.remainder`)
    pub fn matched(&self) -> &str {
        let byte_offset = self.input.len() - self.remainder.len();
        &self.input[..byte_offset]
    }

    /// Merges this error with another [`Error`] from an optional branch of parsing
    ///
    /// The resulting error is the one with smallest remainder string slice, as that is assumed to be more specific and thus helpful.
    ///
    /// Without this method, it would be impossible to retain error information within combinators that can succeed despite errors,
    /// e.g. [`.many(0..)`], [`.optional()`] and [`.or()`]
    ///
    /// [`.many(0..)`]: crate::combinator::many
    /// [`.optional()`]: crate::combinator::optional
    /// [`.or()`]: crate::combinator::or
    pub fn merge(self, other: Error<'i>) -> Error<'i> {
        let mine = self.remainder.len();
        let theirs = other.remainder.len();

        // TODO: consider smarter heuristics and remember to merge any other metadata that gets added!
        if mine < theirs {
            self
        } else {
            other
        }
    }

    /// This returns an ErrorOwned built from this Error
    pub fn to_owned(&self) -> ErrorOwned {
        ErrorOwned {
            reason: self.reason,
            remainder: self.remainder.to_string(),
            input: self.input.to_string(),
        }
    }
}

/// This is a simple "all the possible errors while parsing" enum.
#[non_exhaustive]
#[derive(PartialEq, Debug, Clone, Copy)]
pub enum ErrorReason {
    /// A lexer did not see the expected input and has failed to match
    NoMatch,

    /// A parser encountered an error when converting to the output type
    FailedConversion,
}

impl<'i> std::error::Error for Error<'i> {}
impl<'i> fmt::Display for Error<'i> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.reason {
            ErrorReason::NoMatch => write!(f, "No Match"),
            ErrorReason::FailedConversion => write!(f, "Failed to convert matched input"),
        }
    }
}

/// This module acts as a sort of targeted prelude for all of the error handling traits used by Parsely.
///
/// These are provided for your convenience to act on [`Result`]s directly instead of peppering your parsers with `.map_err(|e| ...)`
///
/// Each trait provides a method of [`Error`] of the same name to the appropriate [`Result`] types.
pub mod result_ext {
    use crate::{Error, ErrorOwned, ParseResult};

    /// This trait is used to extend [`Result<T, parsely::Error>`]
    pub trait ResultExtParselyError<'i, O> {
        /// Calls `.offset()` on the parsely::Error inside
        fn offset(self, input: &'i str) -> Self;

        /// Calls `.to_owned()` on the parsely::Error inside
        fn own_err(self) -> Result<(O, &'i str), ErrorOwned>;

        /// Calls `.merge()` on the parsely::Error inside
        fn merge(self, other: Error<'i>) -> Result<(O, &'i str), Error<'i>>;
    }

    impl<'i, O> ResultExtParselyError<'i, O> for ParseResult<'i, O> {
        fn offset(self, input: &'i str) -> Self {
            self.map_err(|e| e.offset(input))
        }

        fn own_err(self) -> Result<(O, &'i str), ErrorOwned> {
            self.map_err(|e| e.to_owned())
        }

        fn merge(self, other: Error<'i>) -> Result<(O, &'i str), Error<'i>> {
            self.map_err(|e| e.merge(other))
        }
    }

    /// This trait used to extend [`Result<T, E>`] with methods to convert E into [`Error`].
    pub trait ResultExtGenericError<'i, O> {
        /// Replaces the error inside with a [`FailedConversion`](crate::ErrorReason::FailedConversion) [`Error`]
        fn fail_conversion(self, input: &'i str) -> Result<O, Error<'i>>;
    }

    impl<'i, O, E> ResultExtGenericError<'i, O> for Result<O, E> {
        fn fail_conversion(self, input: &'i str) -> Result<O, Error<'i>> {
            self.map_err(|_| Error::failed_conversion(input))
        }
    }
}

/// An owned version of [`Error`].
///
/// This is useful when a trait does not allow specifying lifetime parameters in an assocaiated Error type.
/// For example, this error is needed when implementing [`FromStr`]!
///
/// # Example
///
/// Impl [`FromStr`] using a parsely Parser
///
/// ```
/// # use std::str::FromStr;
/// use parsely::{ErrorOwned, Lex, Parse, ParseResult};
///
/// # const _: &str = stringify! {
/// struct Foo {
///       ...
/// }
/// # };
///
/// #
/// # struct Foo {}
/// #
///
/// fn parser(input: &str) -> ParseResult<'_, Foo> {
/// # const _: &str = stringify! {
///     ...
/// # };
/// # parsely::token("...").map(|_| Foo {}).parse(input)
/// }
///
/// impl FromStr for Foo {
///     type Err = parsely::ErrorOwned;
///
///     fn from_str(s: &str) -> Result<Self, Self::Err> {
///         // ? converts the Error into an ErrorOwned for us
///         let (foo, _) = parser.parse(s)?;
///         Ok(foo)
///     }
/// }
/// ```
///
/// [`FromStr`]: std::str::FromStr
#[derive(PartialEq, Debug)]
pub struct ErrorOwned {
    reason: ErrorReason,
    remainder: String,
    input: String,
}

impl<'i> From<Error<'i>> for ErrorOwned {
    fn from(value: Error<'i>) -> Self {
        value.to_owned()
    }
}

impl std::error::Error for ErrorOwned {}
impl fmt::Display for ErrorOwned {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let error = Error {
            reason: self.reason,
            remainder: &self.remainder,
            input: &self.input,
        };

        error.fmt(f)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::*;

    fn assert_matched<T: fmt::Debug>(error: &Result<T, Error>, expected: &str) {
        let error = error.as_ref().expect_err("no error");
        assert_eq!(error.matched(), expected);
    }

    fn assert_remainder<T: fmt::Debug>(error: &Result<T, Error>, expected: &str) {
        let error = error.as_ref().expect_err("no error");
        assert_eq!(error.remainder, expected);
    }

    fn assert_input<T: fmt::Debug>(error: &Result<T, Error>, expected: &str) {
        let error = error.as_ref().expect_err("no error");
        assert_eq!(error.input, expected);
    }

    fn assert_display<T: fmt::Debug>(error: &Result<T, Error>, expected: &str) {
        let error = error.as_ref().expect_err("no error");
        assert_eq!(error.to_string(), expected.to_string());
    }

    #[test]
    fn test_token_error() {
        let error = "foo".lex("bar");

        assert_matched(&error, "");
        assert_remainder(&error, "bar");
        assert_input(&error, "bar");
        // TODO!: update Display impl
        assert_display(&error, "No Match");
    }
}
