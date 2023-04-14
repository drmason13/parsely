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

use combinators::{or, then, Or, Then};
pub use parsers::*;

#[doc(hidden)]
#[cfg(test)]
pub(crate) mod test_utils;

/// This trait is implemented by all Parsely parsers.
///
/// The [`Parser::parse`] method returns a [`ParseResult`] which contains the output of the parser and the remaining input.
///
/// # Map parser output to a new type
///
/// The output of most parsers will be `&str`, the same type as the input.
///
/// To map the output to a different type you can use the [`ParseResult::map`] or [`ParseResult::try_map`] methods which accept a closure to convert from &str to any type.
///
/// Some built in parsers accept a generic argument of a type to map the output to for you. For example [`parsers::int`] and [`parsers::number`].
pub trait Parser: Sized {
    fn parse<'a>(&mut self, input: &'a str) -> ParseResult<'a>;

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
    /// use parsely::{char, token, Parser};
    ///
    /// let mut for_or_bar = token("foo").or(token("bar"));
    ///
    /// let foo = for_or_bar.parse("foobarbaz");
    ///
    /// assert_eq!(Some("foo"), foo.output());
    /// assert_eq!("barbaz", foo.remaining());
    ///
    /// let bar = for_or_bar.parse("barbaz");
    ///
    /// assert_eq!(Some("bar"), bar.output());
    /// assert_eq!("baz", bar.remaining());
    ///
    /// // `or` can be chained multiple times:
    ///
    /// let mut whitespace = char(' ')
    ///     .or(char('\t'))
    ///     .or(char('\n'))
    ///     .or(char('\r'));
    /// ```
    ///
    /// Note that there is a whitespace parser available, see [`parsers::ws`]
    fn or<P: Parser>(self, parser: P) -> Or<Self, P> {
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
    /// use parsely::{char, hex, Parser};
    ///
    /// let mut hex_color = char('#').then(hex());
    ///
    /// let result = hex_color.parse("#C0FFEE");
    ///
    /// assert_eq!(result.output(), Some("#C0FFEE"));
    /// assert_eq!(result.remaining(), "");
    /// ```
    fn then<P: Parser>(self, parser: P) -> Then<Self, P> {
        then(self, parser)
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct ParseResult<'a> {
    output: Option<&'a str>,
    remaining: &'a str,
}

impl<'a> ParseResult<'a> {
    pub fn new(output: Option<&'a str>, remaining: &'a str) -> Self {
        ParseResult { output, remaining }
    }

    pub fn remaining(&self) -> &str {
        self.remaining
    }

    pub fn output(&self) -> Option<&str> {
        self.output
    }

    pub fn or(self, parser: &mut impl Parser) -> Self {
        match self.output {
            Some(_) => self,
            None => parser.parse(self.remaining),
        }
    }

    pub fn then(self, parser: &mut impl Parser) -> (Self, Option<Self>) {
        match self.output {
            Some(_) => {
                let right = parser.parse(self.remaining);
                (self, Some(right))
            }
            None => (self, None),
        }
    }

    pub fn map<F, O>(self, f: F) -> (Option<O>, Self)
    where
        F: FnMut(&'a str) -> O,
    {
        let output = self.output.map(f);
        (output, self)
    }

    pub fn try_map<F, O, E>(self, f: F) -> Option<Result<O, E>>
    where
        F: FnMut(&'a str) -> Result<O, E>,
    {
        self.output.map(f)
    }
}
