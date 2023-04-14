use std::fmt;

use crate::{ParseResult, Parser};

pub struct Then<L: Parser, R: Parser> {
    left: L,
    right: R,
}

pub fn then<L, R>(left: L, right: R) -> Then<L, R>
where
    L: Parser,
    R: Parser,
{
    Then { left, right }
}

impl<L, R> Parser for Then<L, R>
where
    L: Parser,
    R: Parser,
{
    fn parse<'a>(&mut self, input: &'a str) -> ParseResult<'a> {
        match self.left.parse(input).then(&mut self.right) {
            (
                ParseResult {
                    output: Some(left),
                    remaining: _,
                },
                Some(ParseResult {
                    output: Some(right),
                    remaining: _,
                }),
            ) => {
                // both parsers matched: split input at end of second output
                let boundary = left.len() + right.len();
                let (output, remaining) = input.split_at(boundary);
                ParseResult {
                    output: Some(output),
                    remaining,
                }
            }
            (
                ParseResult {
                    output: Some(_left),
                    remaining: _,
                },
                Some(ParseResult {
                    output: None,
                    remaining: _,
                }),
            ) => {
                // first parser matched, second parser didn't: do not match
                ParseResult {
                    output: None,
                    remaining: input,
                }
            }
            (_left, None) => {
                // first parser executed and did not match: do not match
                ParseResult {
                    output: None,
                    remaining: input,
                }
            }
            _ => unreachable!("Then must only execute the second parser if the first one matches"),
        }
    }
}

impl<L: Parser, R: Parser> fmt::Display for Then<L, R>
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
    use crate::test_utils::*;
    use crate::{char, token};

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
