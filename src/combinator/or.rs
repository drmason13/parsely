use std::fmt;

use crate::{Lex, LexResult, Parse, ParseResult};

#[derive(Clone)]
pub struct Or<L, R> {
    left: L,
    right: R,
}

/// Creates a parser that will attempt to parse with the left parser, and if it fails try to parse with the right parser.
///
/// This short-circuits such that the right parser isn't attempted if the left one matches.
pub fn or<L, R>(left: L, right: R) -> Or<L, R> {
    Or { left, right }
}

impl<L, R, O> Parse for Or<L, R>
where
    L: Parse<Output = O>,
    R: Parse<Output = O>,
{
    type Output = O;

    fn parse<'i>(&self, input: &'i str) -> ParseResult<'i, O> {
        self.left.parse(input).or_else(|_| self.right.parse(input))
    }
}

impl<L, R> Lex for Or<L, R>
where
    L: Lex,
    R: Lex,
{
    fn lex<'i>(&self, input: &'i str) -> LexResult<'i> {
        self.left.lex(input).or_else(|_| self.right.lex(input))
    }
}

impl<L, R> fmt::Debug for Or<L, R>
where
    L: fmt::Debug,
    R: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Or({:?}, {:?})", self.left, self.right)
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
            "token or char",
            or(token("foo"), char('X')),
            &[
                ("foob", Some("foo"), "b"), //
                ("foobcd", Some("foo"), "bcd"),
                ("XYZ", Some("X"), "YZ"),
                ("Xfoo", Some("X"), "foo"),
                ("zzz", None, "zzz"),
            ],
        );
    }

    #[test]
    fn nested() {
        test_lexer_batch(
            "(foo then bar) or (baz then quux)",
            or(
                token("foo").then(token("bar")), //
                token("baz").then(token("quux")),
            ),
            &[
                ("foobar", Some("foobar"), ""),
                ("bazquux", Some("bazquux"), ""),
                ("foobaz", None, "foobaz"),
            ],
        );

        test_lexer_batch(
            "(foo or (bar or baz))",
            or(
                token("foo"), //
                or(token("bar"), token("baz")),
            ),
            &[
                ("foobar", Some("foo"), "bar"),
                ("bazquux", Some("baz"), "quux"),
                ("foobaz", Some("foo"), "baz"),
                ("quuxquux", None, "quuxquux"),
            ],
        );

        test_lexer_batch(
            "((foo or bar) or baz)",
            or(
                or(token("foo"), token("bar")), //
                token("baz"),
            ),
            &[
                ("foobar", Some("foo"), "bar"),
                ("foofoobarbar", Some("foo"), "foobarbar"),
                ("bazquux", Some("baz"), "quux"),
                ("foobaz", Some("foo"), "baz"),
                ("quuxquux", None, "quuxquux"),
            ],
        );
    }
}
