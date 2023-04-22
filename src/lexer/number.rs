use std::fmt;

use crate::lexer::char;
use crate::{Lex, LexResult};

#[derive(Clone)]
pub struct Digit {
    radix: u32,
}

impl Lex for Digit {
    fn lex<'i>(&mut self, input: &'i str) -> LexResult<'i> {
        if let Some(c) = input.chars().next() {
            if c.is_digit(self.radix) {
                Ok(input.split_at(c.len_utf8()))
            } else {
                Err(crate::Error::NoMatch)
            }
        } else {
            Err(crate::Error::NoMatch)
        }
    }
}

pub fn digit() -> Digit {
    Digit { radix: 10 }
}

impl Digit {
    /// Create a new Digit lexer that matches digits with the base n.
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

/// A lexer that parses an integer, i.e. one or more base 10 digits with or without a leading '-' indicating the sign.
///
/// To lex unsigned integers that forbid the leading '-' consider using:
//TODO: * [`uint()`] which will lex only base 10 digits
//TODO: * [`digit(10)`] which is the implementation of [`uint()`]
///
/// To lex decimals consider using:
/// * [`float()`] which will lex only decimals
/// * [`number()`] which will lex integers or decimals
///
pub fn int() -> impl Lex + fmt::Debug {
    char('-').many(0..=1).then(digit().many(1..))
}

/// A lexer that parses an hexadecimal character, i.e. one or more base 16 digits.
///
/// No leading `0x` or other hex notation in the input is accepted.
///
/// To lex decimals consider using:
/// * [`float()`] which will lex only decimals
/// * [`number()`] which will lex integers or decimals
///
/// # Examples
///
/// # Note
///
//TODO: This lexer will not transform its output into another type, but this can be done using [`Lex::map`].
pub fn hex() -> Digit {
    Digit { radix: 16 }
}

// To return impl Lexr or the specific lexer?
// `impl Lexr` encapsulates the implementation so we can change it without breaking semver, but might cause type shenanigans
// the specific lexer is a mouthful, not "simple" and easily leads to breaking semver, but might reduce type shenanigans?
pub fn float() -> impl Lex + fmt::Debug {
    int() //
        .then(char('.'))
        .then(digit().many(0..))
}

pub fn number() -> impl Lex + fmt::Debug {
    float().or(int())
}

impl fmt::Debug for Digit {
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
        test_lexer_batch(
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
    fn test_hex() {
        test_lexer_batch(
            "hex works",
            hex(),
            &[
                ("", None, ""), //
                ("123", Some("1"), "23"),
                ("abc", Some("a"), "bc"),
                ("ABC", Some("A"), "BC"),
                ("GHI", None, "GHI"),
            ],
        );

        test_lexer_batch(
            "hex many works",
            hex().many(1..),
            &[
                ("", None, ""), //
                ("123", Some("123"), ""),
                ("abc", Some("abc"), ""),
                ("ABC", Some("ABC"), ""),
                ("GHI", None, "GHI"),
                ("EFG", Some("EF"), "G"),
                ("C0FFEE", Some("C0FFEE"), ""),
            ],
        );
    }

    #[test]
    fn parsing() {
        test_lexer_batch(
            "int matches base 10 digits",
            int(),
            &[
                ("abc", None, "abc"), //
                ("123", Some("123"), ""),
                ("1.23", Some("1"), ".23"),
            ],
        );

        test_lexer_batch(
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

        test_lexer_batch(
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
