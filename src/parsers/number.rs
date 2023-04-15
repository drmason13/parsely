use std::fmt;

use crate::parsers::char;
use crate::{ParseResult, Parser};

pub struct Digit {
    radix: u32,
}

impl Parser for Digit {
    fn parse<'a>(&mut self, input: &'a str) -> ParseResult<'a> {
        if let Some(c) = input.chars().next() {
            if c.is_digit(self.radix) {
                let (output, remaining) = input.split_at(c.len_utf8());
                ParseResult::new(Some(output), remaining)
            } else {
                ParseResult::new(None, input)
            }
        } else {
            ParseResult::new(None, input)
        }
    }
}

pub fn digit() -> Digit {
    Digit { radix: 10 }
}

impl Digit {
    /// Create a new Digit parser that matches digits with the base n.
    ///
    /// # Examples
    ///
    /// Basic usage:
    ///
    /// ```
    /// use parsely::digit;
    ///
    /// let base_10 = digit();
    ///
    /// let base_32 = digit().base(32);
    /// ```
    pub fn base(&self, n: u32) -> Digit {
        Digit { radix: n }
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
pub fn int() -> impl Parser + fmt::Display {
    char('-').many(0..=1).then(digit().many(1..))
}

/// A parser that parses an hexadecimal character, i.e. one or more base 16 digits.
///
/// No leading `0x` or other hex notation in the input is accepted.
///
/// To parse decimals consider using:
/// * [`float()`] which will parse only decimals
/// * [`number()`] which will parse integers or decimals
///
/// # Examples
///
/// # Note
///
/// This parser will not transform its output into another type, but this can be done using [`ParseResult::map`].
pub fn hex() -> Digit {
    Digit { radix: 16 }
}

// To return impl Parser or the specific parser?
// `impl Parser` encapsulates the implementation so we can change it without breaking semver, but might cause type shenanigans
// the specific parser is a mouthful, not "simple" and easily leads to breaking semver, but might reduce type shenanigans?
pub fn float() -> impl Parser + fmt::Display {
    int() //
        .then(char('.'))
        .then(digit().many(0..))
}

pub fn number() -> impl Parser + fmt::Display {
    float().or(int())
}

impl fmt::Display for Digit {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "digit({})", self.radix)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_utils::*;

    #[test]
    fn digit_char_find() {
        let actual = "123a".chars().find(|c| !c.is_ascii_digit());
        assert_eq!(actual, Some('a'), "can find a");

        let actual = "".chars().find(|c| !c.is_ascii_digit());
        assert_eq!(actual, None, "empty finds nothing");
    }

    #[test]
    fn test_digit() {
        test_parser_batch(
            "digit works",
            digit(),
            &[
                ("", None, ""), //
                ("123", Some("1"), "23"),
                ("abc", None, "abc"),
            ],
        );
    }

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
                ("12.A3", Some("12."), "A3"),
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
                ("12.A3", Some("12."), "A3"),
            ],
        );
    }
}
