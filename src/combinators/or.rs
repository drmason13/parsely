use std::fmt;

use crate::{Parse, ParseResult};

pub struct Or<L: Parse, R: Parse> {
    left: L,
    right: R,
}

/// Attempt to parse with the left parser, and if it fails try to parse with the right parser.
///
/// This short-circuits such that the right parser isn't attempted if the left one matches.
pub fn or<L, R>(left: L, right: R) -> Or<L, R>
where
    L: Parse,
    R: Parse,
{
    Or { left, right }
}

impl<L, R> Parse for Or<L, R>
where
    L: Parse,
    R: Parse,
{
    fn parse<'i>(&mut self, input: &'i str) -> ParseResult<'i> {
        self.left.parse(input).or_else(|_| self.right.parse(input))
    }
}

impl<L: Parse, R: Parse> fmt::Display for Or<L, R>
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
    use crate::test_utils::*;
    use crate::{char, token};

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
