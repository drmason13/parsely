use std::fmt;

use crate::Parser;

pub struct Or<L: Parser, R: Parser> {
    left: L,
    right: R,
}

pub fn or<L, R>(left: L, right: R) -> Or<L, R>
where
    L: Parser,
    R: Parser,
{
    Or { left, right }
}

impl<L, R> Parser for Or<L, R>
where
    L: Parser,
    R: Parser,
{
    fn parse<'a>(&mut self, input: &'a str) -> crate::ParseResult<'a> {
        self.left.parse(input).or(&mut self.right)
    }
}

impl<L: Parser, R: Parser> fmt::Display for Or<L, R>
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
}
