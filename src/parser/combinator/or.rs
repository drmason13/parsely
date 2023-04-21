use std::{fmt, marker::PhantomData};

use crate::{Parse, ParseResult};

pub struct Or<L: Parse, R: Parse, O> {
    left: L,
    right: R,
    phantom: PhantomData<O>,
}

/// Creates a parser that will attempt to parse with the left parser, and if it fails try to parse with the right parser.
///
/// This short-circuits such that the right parser isn't attempted if the left one matches.
pub fn or<L, R, O>(left: L, right: R) -> Or<L, R, O>
where
    L: Parse,
    R: Parse,
{
    Or {
        left,
        right,
        phantom: PhantomData,
    }
}

impl<L, R, O> Parse for Or<L, R, O>
where
    L: Parse,
    R: Parse,
{
    fn parse<'i>(&mut self, input: &'i str) -> ParseResult<'i, O> {
        self.left.parse(input).or_else(|_| self.right.parse(input))
    }
}

impl<L: Parse, R: Parse, O> fmt::Display for Or<L, R, O>
where
    L: fmt::Display,
    R: fmt::Display,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "({} or {})", self.left, self.right)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lexer::{char, token};
    use crate::test_utils::*;

    #[test]
    fn parsing() {
        test_parser_batch(
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
        test_parser_batch(
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

        test_parser_batch(
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

        test_parser_batch(
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
