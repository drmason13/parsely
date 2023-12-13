use std::fmt;

use crate::{End, Lex, LexResult, Parse, ParseResult};

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
/// To run a parser followed by a lexer, see [`Parse::then_skip()`].
///
/// To run a lexer followed by a parser, see [`Lex::skip_then()`].
///
/// This combinator can be chained using [`Parse::then()`] or [`Lex::then()`].
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

impl<L> Parse for Then<L, End>
where
    L: Parse,
{
    type Output = <L as Parse>::Output;

    fn parse<'i>(&self, input: &'i str) -> ParseResult<'i, Self::Output> {
        let (left, remaining) = self.left.parse(input)?;
        let (_, remaining) = self.right.lex(remaining)?;

        Ok((left, remaining))
    }
}

impl<L: Lex, R: Lex> Lex for Then<L, R> {
    fn lex<'i>(&self, input: &'i str) -> LexResult<'i> {
        let (left, left_remaining) = self.left.lex(input)?;
        let (right, right_remaining) = self.right.lex(left_remaining)?;

        let boundary = left.len() + right.len();
        let (matched, remaining) = input.split_at(boundary);

        // Enforcing the fundamental law of parsely lexing
        debug_assert_eq!(
            right_remaining, remaining,
            "the fundamental law of parsely lexing has been broken!"
        );

        Ok((matched, remaining))
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
    use crate::{int, Lex, Parse};

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

    #[derive(Debug, PartialEq)]
    pub enum Color {
        Red,
    }

    #[test]
    fn then_swap() -> Result<(), crate::Error> {
        let red = token("red").map(|_| Color::Red);
        let (output, _) = int::<u8>().pad().then(red).swap().parse("4 red")?;

        assert_eq!(output, (Color::Red, 4));

        Ok(())
    }
}
