//! The error returned when a parser or lexer does not match successfully.
//!
//! Parsely's error handling strategy is currently unstable. Expect these types to change.

use std::fmt;

/// The [`Error`] type returned by both [`parse`] and [`lex`] methods.
///
/// Errors in parsely don't directly capture a Span like most parsing libraries.
///
/// They simply store two slices of the original [`&str`](str) input: `remaining` and `input`
///
/// This means it is cheap to create and clone, once the parsing is finished you will often want to convert this error type into a [`parsely::InProgressError`] by [`complete()`]-ing it.
///
/// [`parse`]: crate::Parse::parse()
/// [`lex`]: crate::Lex::lex()
/// [`complete()`]: crate::InProgressError::complete
#[derive(PartialEq, Debug)]
pub struct InProgressError<'i> {
    /// The reason for the error
    pub reason: ErrorReason,

    /// The remaining unparsed input
    pub remaining: &'i str,

    /// The input to the first parser to run, the *original* input
    pub input: &'i str,
}

pub enum AnyError<'i> {
    Cheap(InProgressError<'i>),
    Annotated {
        error: InProgressError<'i>,
        message: Vec<String>,
    },
    Complete(Error),
}

impl<'i> From<InProgressError<'i>> for AnyError<'i> {
    fn from(value: InProgressError<'i>) -> Self {
        AnyError::Cheap(value)
    }
}

impl<'i> From<Error> for AnyError<'i> {
    fn from(value: Error) -> Self {
        AnyError::Complete(value)
    }
}

pub fn no_match(input: &str) -> InProgressError<'_> {
    InProgressError {
        input,
        remaining: input,
        reason: ErrorReason::NoMatch,
    }
}

pub fn failed_conversion(input: &str) -> InProgressError<'_> {
    InProgressError {
        input,
        remaining: input,
        reason: ErrorReason::FailedConversion,
    }
}

impl<'i> InProgressError<'i> {
    /// Create a new error at the point that a lexer failed to match the input
    ///
    /// See [`ErrorReason::NoMatch`]
    pub fn no_match(input: &'i str) -> Self {
        InProgressError {
            input,
            remaining: input,
            reason: ErrorReason::NoMatch,
        }
    }

    /// Create a new error at the point that a parser failed to convert matched input into the output type
    ///
    /// See [`ErrorReason::FailedConversion`]
    pub fn failed_conversion(input: &'i str) -> Self {
        InProgressError {
            input,
            remaining: input,
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
    /// Note: this is exactly from the start of the input up until the point where the parser failed to match (`error.remaining`)
    pub fn matched(&self) -> &str {
        let byte_offset = self.input.len() - self.remaining.len();
        &self.input[..byte_offset]
    }

    /// Merges this error with another [`Error`] from an optional branch of parsing
    ///
    /// The resulting error is the one with smallest remaining string slice, as that is assumed to be more specific and thus helpful.
    ///
    /// Without this method, it would be impossible to retain error information within combinators that can succeed despite errors,
    /// e.g. [`.many(0..)`], [`.optional()`] and [`.or()`]
    ///
    /// [`.many(0..)`]: crate::combinator::many
    /// [`.optional()`]: crate::combinator::optional
    /// [`.or()`]: crate::combinator::or
    pub fn merge(self, other: InProgressError<'i>) -> InProgressError<'i> {
        let mine = self.remaining.len();
        let theirs = other.remaining.len();

        // TODO: consider smarter heuristics and remember to merge any other metadata that gets added!
        if mine < theirs {
            self
        } else {
            other
        }
    }

    /// This returns an ErrorOwned built from this Error
    pub fn complete(&self) -> Error {
        Error {
            reason: self.reason,
            remaining: self.remaining.to_string(),
            input: self.input.to_string(),
        }
    }
}

/// A simple "all the possible errors while parsing" enum
///
/// It is included as a field in [`Error`].
#[non_exhaustive]
#[derive(PartialEq, Debug, Clone, Copy)]
pub enum ErrorReason {
    /// A lexer did not see the expected input and has failed to match
    ///
    /// You can construct an [`Error`] with this reason using [`Error::no_match()`]
    NoMatch,

    /// A parser encountered an error when converting to the output type
    FailedConversion,
}

impl<'i> std::error::Error for InProgressError<'i> {}
impl<'i> fmt::Display for InProgressError<'i> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.reason {
            ErrorReason::NoMatch => write!(f, "No Match"),
            ErrorReason::FailedConversion => write!(f, "Failed to convert matched input"),
        }
    }
}

/// This module contains two "ResultExt" traits for error handling.
///
/// They provide methods on [`Result`]s directly to reduce the need to pepper your parsers with `.map_err(|e| ...)`
pub mod result_ext {
    use crate::{Error, InProgressError, ParseResult};

    /// This trait is used to extend [`Result<T, parsely::InProgressError>`]
    pub trait ResultExtParselyError<'i, O> {
        /// Calls [`.offset()`] on the parsely::InProgressError inside
        ///
        /// [`.offset()`]: Error::offset()
        fn offset(self, input: &'i str) -> Self;

        /// Calls [`.complete()`] on the parsely::InProgressError inside
        ///
        /// [`.complete()`]: Error::complete()
        fn complete(self) -> Result<(O, &'i str), Error>;

        /// Calls [`.merge()`] on the parsely::InProgressError inside
        ///
        /// [`.merge()`]: Error::merge()
        fn merge(self, other: InProgressError<'i>) -> Result<(O, &'i str), InProgressError<'i>>;
    }

    impl<'i, O> ResultExtParselyError<'i, O> for ParseResult<'i, O> {
        fn offset(self, input: &'i str) -> Self {
            self.map_err(|e| e.offset(input))
        }

        fn complete(self) -> Result<(O, &'i str), Error> {
            self.map_err(|e| e.complete())
        }

        fn merge(self, other: InProgressError<'i>) -> Result<(O, &'i str), InProgressError<'i>> {
            self.map_err(|e| e.merge(other))
        }
    }

    /// This trait used to extend [`Result<T, E>`] with methods to convert `E` into [`Error`].
    pub trait ResultExtGenericError<'i, O> {
        /// Replaces the error inside with a [`FailedConversion`](crate::ErrorReason::FailedConversion) [`Error`]
        fn fail_conversion(self, input: &'i str) -> Result<O, InProgressError<'i>>;
    }

    impl<'i, O, E> ResultExtGenericError<'i, O> for Result<O, E> {
        fn fail_conversion(self, input: &'i str) -> Result<O, InProgressError<'i>> {
            self.map_err(|_| InProgressError::failed_conversion(input))
        }
    }
}

/// The output of a *complete*, but failed, parsing attempt. Where *complete* is determined by the end user.
///
/// All Parsely traits return [`InProgressError`]s since as a library, we cannot know which parsers will be at the top level.
///
/// You can call [`.complete()`] on an in progress error to convert it into [`Error`].
///
/// # Example
///
/// Impl [`FromStr`] using a parsely Parser.
///
/// When implementing the [`FromStr`] trait, the associated Error type has no lifetime parameter (this is very common!).
///
/// It is only possible to impl [`FromStr`] by using [`Error`] as the associated Error type because [`InProgressError`] requires a lifetime parameter.
///
/// ```
/// # use std::str::FromStr;
/// use parsely::{Lex, Parse, ParseResult};
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
///     type Err = parsely::Error;
///
///     fn from_str(s: &str) -> Result<Self, Self::Err> {
///         // ? converts the InProgressError into an Error for us
///         let (foo, _) = parser.parse(s)?;
///         Ok(foo)
///     }
/// }
/// ```
///
/// [`FromStr`]: std::str::FromStr
/// [`.complete()`]: InProgressError::complete()
#[derive(PartialEq, Debug)]
pub struct Error {
    /// The reason for the error
    pub reason: ErrorReason,

    /// The start and end of the input that was matched before erroring
    ///
    /// this should start at line 0, column 0 - the start of the input
    pub matched: Span,

    /// The start and end of the remaining unparsed input
    ///
    /// this should end at the end of the input
    pub remaining: Span,

    /// Where in the input the error occurred
    ///
    /// This should be equal to the start position of remaining
    pub failed_at: Position,
}

/// Describes the start and end of a &str slice of some input
#[derive(Clone, PartialEq, Debug)]
pub struct Span {
    pub start: Position,
    pub end: Position,
}

/// Describes a position in a piece of text by line and column count, columns are counted using grapheme clusters
#[derive(Clone, PartialEq, Debug)]
pub struct Position {
    pub line: usize,
    pub column: usize,
}

impl Position {
    /// Calculates the [`Position`] at the *end* of the given input text
    pub fn end(input: &str) -> Self {
        input.graphemes().fold(todo!())
    }
}

impl Error {
    /// Returns the part of the input that was matched overall before failure
    pub fn matched(&self, input: &str) -> &str {
        &self.input[..byte_offset]
    }
}

impl<'i> From<InProgressError<'i>> for Error {
    fn from(value: InProgressError<'i>) -> Self {
        value.complete()
    }
}

impl std::error::Error for Error {}
impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let error = InProgressError {
            reason: self.reason,
            remaining: &self.remaining,
            input: &self.input,
        };

        error.fmt(f)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::*;

    fn assert_matched<T: fmt::Debug>(error: &Result<T, InProgressError>, expected: &str) {
        let error = error.as_ref().expect_err("no error");
        assert_eq!(error.matched(), expected);
    }

    fn assert_remaining<T: fmt::Debug>(error: &Result<T, InProgressError>, expected: &str) {
        let error = error.as_ref().expect_err("no error");
        assert_eq!(error.remaining, expected);
    }

    fn assert_input<T: fmt::Debug>(error: &Result<T, InProgressError>, expected: &str) {
        let error = error.as_ref().expect_err("no error");
        assert_eq!(error.input, expected);
    }

    fn assert_display<T: fmt::Debug>(error: &Result<T, InProgressError>, expected: &str) {
        let error = error.as_ref().expect_err("no error");
        assert_eq!(error.to_string(), expected.to_string());
    }

    #[test]
    fn test_token_error() {
        let error = "foo".lex("bar");

        assert_matched(&error, "");
        assert_remaining(&error, "bar");
        assert_input(&error, "bar");
        // TODO!: update Display impl
        assert_display(&error, "No Match");
    }
}
