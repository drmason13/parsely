use std::fmt;

use crate::parsers::char;
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

pub fn digit(radix: u32) -> Digit {
    Digit {
        name: "digit",
        radix,
    }
}

/// A parser that parses an integer, i.e. one or more base 10 digits with or without a leading '-' indicating the sign.
///
/// To parse unsigned integers that forbid the leading '-' consider using:
/// * [`uint()`] which will parse only base 10 digits
/// * [`digit(10)`] which is the implementation of [`uint()`]
///
/// To parse decimals consider using:
/// * [`float()`] which will parse only decimals
/// * [`number()`] which will parse integers or decimals
///
/// # Note
///
/// This parser will transform its output into
pub fn int() -> Digit {
    Digit {
        name: "int",
        radix: 10,
    }
}

/// A parser that parses an hexadecimal character, i.e. one or more base 16 digits.
///
/// To parse decimals consider using:
/// * [`float()`] which will parse only decimals
/// * [`number()`] which will parse integers or decimals
///
/// # Note
///
/// This parser will not transform its output into another type, but this can be done using [`Parser::map`].
pub fn hex() -> Digit {
    Digit {
        name: "hex",
        radix: 16,
    }
}

pub fn float() -> impl Parser + fmt::Display {
    int() //
        .then(char('.'))
        .then(int())
}

pub fn number() -> impl Parser + fmt::Display {
    float().or(int())
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
            "int matches base 10 digits",
            int(),
            &[
                ("abc", None, "abc"), //
                ("123", Some("123"), ""),
                ("1.23", Some("1"), ".23"),
            ],
        );

        test_parser_batch(
            "float matches only decimals",
            float(),
            &[
                ("12.6", Some("12.6"), ""),
                ("12.", Some("12."), ""),
                ("123", None, "123"),
                ("12.3A", Some("12.3"), "A"),
                ("12.A3", None, "12.A3"),
                ("12.0.1", Some("12.0"), ".1"),
            ],
        );

        test_parser_batch(
            "number matches base 10 digits or decimals",
            number(),
            &[
                ("12.6", Some("12.6"), ""),
                ("12.", Some("12."), ""),
                ("123", Some("123"), ""),
                ("12.3A", Some("12.3"), "A"),
                ("12.A3", Some("12"), ".A3"),
            ],
        );
    }
}
