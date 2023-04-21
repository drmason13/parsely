use std::fmt;

use crate::{char, digit};
use crate::{Parse, ParseError, ParseResult};

/// A parser that parses an integer, i.e. one or more base 10 digits with or without a leading '-' indicating the sign.
///
/// To parse unsigned integers that forbid the leading '-' consider using:
//TODO: * [`uint()`] which will parse only base 10 digits
//TODO: * [`digit(10)`] which is the implementation of [`uint()`]
///
/// To parse decimals consider using:
/// * [`float()`] which will parse only decimals
/// * [`number()`] which will parse integers or decimals
///
pub fn int<T>() -> impl Parse<Output = T> + fmt::Display {
    char('-')
        .many(0..=1)
        .then(digit().many(1..))
        .map(|n| n.parse())
}

// To return impl Parser or the specific parser?
// `impl Parser` encapsulates the implementation so we can change it without breaking semver, but might cause type shenanigans
// the specific parser is a mouthful, not "simple" and easily leads to breaking semver, but might reduce type shenanigans?
pub fn float() -> impl Parse + fmt::Display {
    int() //
        .then(char('.'))
        .then(digit().many(0..))
}

pub fn number() -> impl Parse + fmt::Display {
    float().or(int())
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
