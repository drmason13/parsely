//!
//! Parsely is a parser combinator library for Rust with the following aims
//!
//! * Simple to use
//! * Well documented built-in parsers
//!
//! While it doesn't prioritise speed, it will often be "fast enough" for a projects that do a little bit of parsing here and there.
//!
//! If parsing speed is important to your application's performance (for example a compiler) then this library isn't meant for you.

pub mod parsers;

#[doc(hidden)]
#[cfg(test)]
pub(crate) mod test_utils;

#[derive(Debug, PartialEq, Eq)]
pub struct ParseOutput<'a> {
    processed: Option<&'a str>,
    remaining: &'a str,
}

impl<'a> ParseOutput<'a> {
    pub fn new(processed: Option<&'a str>, remaining: &'a str) -> Self {
        ParseOutput {
            processed,
            remaining,
        }
    }

    pub fn and(self, mut parser: impl Parser) -> Self {
        match self.processed {
            Some(next) => parser.parse(next),
            None => self,
        }
    }

    pub fn pipe(self, mut parser: impl Parser) -> Self {
        parser.parse_piped(self)
    }

    pub fn map<F, O>(self, f: F) -> Option<O>
    where
        F: FnMut(&'a str) -> O,
    {
        self.processed.map(f)
    }
}

pub trait Parser {
    fn parse<'a>(&mut self, input: &'a str) -> ParseOutput<'a>;

    fn parse_piped<'a>(&mut self, input: ParseOutput<'a>) -> ParseOutput<'a> {
        let input = input.remaining;
        self.parse(input)
    }

    /// The user friendly name of the parser, will be printed in error messages
    fn name(&self) -> &'static str {
        "token"
    }
}
