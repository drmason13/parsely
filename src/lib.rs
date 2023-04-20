#![deny(rustdoc::broken_intra_doc_links)]

//!
//! Parsely is a parser combinator library for Rust with the following aims
//!
//! * Simple to use
//! * Well documented built-in parsers
//!
//! While it doesn't prioritise speed, it will often be "fast enough" for a projects that do a little bit of parsing here and there.
//!
//! If parsing speed is important to your application's performance (for example a compiler) then this library isn't meant for you.

pub mod combinators;
pub mod parsers;

use std::ops::RangeBounds;

pub use combinators::*;
pub use parsers::*;

#[non_exhaustive]
#[derive(Debug, PartialEq)]
pub enum ParseError {
    NoMatch,
}

pub type ParseResult<'i> = Result<(&'i str, &'i str), ParseError>;

#[doc(hidden)]
#[cfg(test)]
pub(crate) mod test_utils;

/// This trait is implemented by all Parsely parsers.
///
/// The [`Parser::parse`] method returns a tuple `(matched, remaining)` of `&str`.
/// First the part of the input successfully matched and then the remaining part of the input that was not matched.
/// The order reads left to right as the parser reads the input, and matches the return order of [`std::str::split_at`].
///
/// # Map parser output to a new type
///
/// The output of most parsers will be `&str`, the same type as the input.
///
/// To map the output to a different type you can use the [`Parse::map`] or [`Parse::try_map`] methods which accept a closure to do the conversion.
///
/// Some built in parsers accept a generic argument of a type to map the output to for you. For example [`parsers::int`] and [`parsers::number`].
pub trait Parse: Sized {
    fn parse<'i>(&mut self, input: &'i str) -> ParseResult<'i>;

    /// Creates a new parser that will attempt to parse with this parser multiple times.
    ///
    /// See [`combinators::Many`] for more details.
    fn many(self, range: impl RangeBounds<usize>) -> Many<Self>
    where
        Self: Sized,
    {
        many(range, self)
    }

    /// Creates a new parser that will attempt to parse with this parser exactly n times.
    ///
    /// See [`combinators::Many`] for more details.
    fn count(self, n: usize) -> Many<Self>
    where
        Self: Sized,
    {
        count(n, self)
    }

    /// Creates a new parser that will attempt to parse with this parser, and if it fails, attempt to parse with the given parser.
    ///
    /// This can be used to build a chain of possible ways to parse the same input.
    ///
    /// At most, one of the parsers will consume input.
    ///
    /// # Examples
    ///
    /// Basic usage:
    ///
    /// ```
    /// use parsely::{char, token, Parse, ParseError};
    ///
    /// let mut for_or_bar = token("foo").or(token("bar"));
    ///
    /// let (output, remaining) = for_or_bar.parse("foobarbaz")?;
    ///
    /// assert_eq!(output, "foo");
    /// assert_eq!(remaining, "barbaz");
    ///
    /// let (output, remaining) = for_or_bar.parse("barbaz")?;
    ///
    /// assert_eq!(output, "bar");
    /// assert_eq!(remaining, "baz");
    ///
    /// // `or` can be chained multiple times:
    ///
    /// let mut whitespace = char(' ')
    ///     .or(char('\t'))
    ///     .or(char('\n'))
    ///     .or(char('\r'));
    ///
    /// # Ok::<(), ParseError>(())
    /// ```
    ///
    /// Note that there is a whitespace parser available, see [`parsers::ws`]
    fn or<P: Parse>(self, parser: P) -> Or<Self, P>
    where
        Self: Sized,
    {
        or(self, parser)
    }

    /// Creates a new parser that applies two parsers in sequence.
    ///
    /// First this parser is run, and then if successful, the remaining input will be fed to the given parser.
    ///
    /// This parser short circuits such that if the first parser does not match, the second one is not attempted.
    ///
    /// Both parsers are required to match for any input to be consumed.
    ///
    /// # Examples
    ///
    /// Basic usage:
    ///
    /// ```
    /// use parsely::{char, hex, Parse, ParseError};
    ///
    /// let mut hex_color = char('#').then(hex().many(1..));
    ///
    /// let (output, remaining) = hex_color.parse("#C0FFEE")?;
    ///
    /// assert_eq!(output, "#C0FFEE");
    /// assert_eq!(remaining, "");
    ///
    /// let result = hex_color.parse("#TEATEA");
    ///
    /// assert_eq!(result, Err(ParseError::NoMatch));
    ///
    /// # Ok::<(), ParseError>(())
    /// ```
    fn then<P: Parse>(self, parser: P) -> Then<Self, P>
    where
        Self: Sized,
    {
        then(self, parser)
    }
}
