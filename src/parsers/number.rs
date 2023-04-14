use std::fmt;

use crate::{ParseResult, Parser};

pub struct Digit {
    name: &'static str,
    radix: u32,
}

impl Parser for Digit {
    fn parse<'a>(&mut self, input: &'a str) -> ParseResult<'a> {
        let mut chars = input.char_indices();

        match chars.find(|(_, c)| !c.is_digit(self.radix)) {
            Some((i, _)) => {
                if i == 0 {
                    // the first char was not a digit
                    ParseResult::new(None, input)
                } else {
                    let (output, remaining) = input.split_at(i);
                    ParseResult::new(Some(output), remaining)
                }
            }
            None => ParseResult::new(Some(input), ""),
        }
    }
}

pub fn int() -> Digit {
    Digit {
        name: "int",
        radix: 10,
    }
}

pub fn number() -> impl Parser {
    Digit {
        name: "int",
        radix: 10,
    }
}

impl fmt::Display for Digit {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_utils::*;

    #[test]
    fn parsing() {
        test_parser_batch(
            "matches base 10 digits",
            int(),
            &[
                ("abc", None, "abc"), //
                ("123", Some("123"), ""),
                ("1.23", Some("1"), ".23"),
            ],
        );
    }
}
