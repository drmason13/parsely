use std::fmt;

use crate::{Lex, LexResult, Parse, ParseResult};

/// This combinator is returned by [`then()`]. See it's documentation for more details.
#[derive(Clone)]
pub struct Then<L, R> {
    left: L,
    right: R,
}

/// This combinator is used to run 2 parsers or 2 lexers sequentially.
///
/// Both must match, if either the left or right item returns an error, [`then()`] fails.
///
/// This combinator can be chained using [`Parse::then()`] or [`Lex::then()`]
pub fn then<L, R>(left: L, right: R) -> Then<L, R> {
    Then { left, right }
}

impl<L, R> Parse for Then<L, R>
where
    L: Parse,
    R: Parse,
{
    type Output = (<L as Parse>::Output, <R as Parse>::Output);

    fn parse<'i>(&self, input: &'i str) -> ParseResult<'i, Self::Output> {
        let (left, remaining) = self.left.parse(input)?;
        let (right, remaining) = self.right.parse(remaining)?;

        let boundary = input.len() - remaining.len();
        let (_, remaining) = input.split_at(boundary);

        Ok(((left, right), remaining))
    }
}

impl<L: Lex, R: Lex> Lex for Then<L, R> {
    fn lex<'i>(&self, input: &'i str) -> LexResult<'i> {
        let (left, remaining) = self.left.lex(input)?;
        let (right, _) = self.right.lex(remaining)?;

        let boundary = left.len() + right.len();
        Ok(input.split_at(boundary))
    }
}

impl<L, R> fmt::Debug for Then<L, R>
where
    L: fmt::Debug,
    R: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Then({:?} -> {:?})", self.left, self.right)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lexer::{char, token};
    use crate::test_utils::*;

    #[test]
    fn parsing() {
        test_lexer_batch(
            "token then char",
            then(token("foo"), char('X')),
            &[
                ("foo123", None, "foo123"), //
                ("fooX123", Some("fooX"), "123"),
                ("X123", None, "X123"),
                ("Xfoo", None, "Xfoo"),
                ("fooX", Some("fooX"), ""),
                ("zzz", None, "zzz"),
            ],
        );
    }
}
