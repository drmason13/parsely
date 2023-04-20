use std::fmt;

use crate::{Lex, LexResult};

pub struct Then<L: Lex, R: Lex> {
    left: L,
    right: R,
}

pub fn then<L, R>(left: L, right: R) -> Then<L, R>
where
    L: Lex,
    R: Lex,
{
    Then { left, right }
}

impl<L, R> Lex for Then<L, R>
where
    L: Lex,
    R: Lex,
{
    fn lex<'i>(&mut self, input: &'i str) -> LexResult<'i> {
        let (left, remaining) = self.left.lex(input)?;
        let (right, _) = self.right.lex(remaining)?;

        let boundary = left.len() + right.len();
        Ok(input.split_at(boundary))
    }
}

impl<L: Lex, R: Lex> fmt::Display for Then<L, R>
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
