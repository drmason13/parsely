use std::fmt;

use crate::{Parse, ParseResult};

pub struct Then<L: Parse, R: Parse> {
    left: L,
    right: R,
}

pub fn then<L, R>(left: L, right: R) -> Then<L, R>
where
    L: Parse,
    R: Parse,
{
    Then { left, right }
}

impl<L, R> Parse for Then<L, R>
where
    L: Parse,
    R: Parse,
{
    fn parse<'i>(&mut self, input: &'i str) -> ParseResult<'i> {
        let (left, remaining) = self.left.parse(input)?;
        let (right, _) = self.right.parse(remaining)?;

        let boundary = left.len() + right.len();
        Ok(input.split_at(boundary))
    }
}

impl<L: Parse, R: Parse> fmt::Display for Then<L, R>
where
    L: fmt::Display,
    R: fmt::Display,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} -> {}", self.left, self.right)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::{char, token};
    use crate::test_utils::*;

    #[test]
    fn parsing() {
        test_parser_batch(
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
