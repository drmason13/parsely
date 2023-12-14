//! [`lexer.optional()`](Lex::optional) will succeed even if `lexer` fails.
//!
//! [`parser.optional()`](Parse::optional) will succeed even if `parser` fails.

use std::fmt;

use crate::{result_ext::*, Lex, Parse, ParseResult};

/// This combinator is returned by [`optional()`]. See it’s documentation for more details.
#[derive(Clone)]
pub struct Optional<T> {
    item: T,
}

impl<T> Lex for Optional<T>
where
    T: Lex,
{
    fn lex<'i>(&self, input: &'i str) -> crate::LexResult<'i> {
        if let Ok((matched, remaining)) = self.item.lex(input).offset(input) {
            Ok((matched, remaining))
        } else {
            Ok(("", input))
        }
    }
}

impl<T> Parse for Optional<T>
where
    T: Parse,
{
    type Output = Option<<T as Parse>::Output>;

    fn parse<'i>(&self, input: &'i str) -> ParseResult<'i, Self::Output> {
        if let Ok((output, remaining)) = self.item.parse(input).offset(input) {
            Ok((Some(output), remaining))
        } else {
            Ok((None, input))
        }
    }
}

/// Makes an optional parser/lexer.
///
/// If the parser or lexer fails, then the error is silenced and the whole input is returned as remaining input.
///
/// This is more conveniently created using the [`Lex::optional`] and [`Parse::optional`] methods.
pub fn optional<T>(item: T) -> Optional<T> {
    Optional { item }
}

impl<T: fmt::Debug> fmt::Debug for Optional<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Optional({:?})", self.item)
    }
}
