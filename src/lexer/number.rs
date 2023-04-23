use std::fmt;

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

/// A lexer that parses an hexadecimal character, i.e. one or more base 16 digits.
///
/// No leading `0x` or other hex notation in the input is accepted.
///
/// As this is a lexer, no type conversion is performed.
///
/// # Examples
///
/// Basic usage:
///
/// ```
/// use parsely::{hex, Lex};
///
/// assert_eq!(("a", "bc"), hex().lex("abc")?);
///
/// assert_eq!(("123abcdef", "g"), hex().many(1..).lex("123abcdefg")?);
/// # Ok::<(), parsely::Error>(())
/// ```
///
/// Convert to u8:
///
/// ```
/// use parsely::{hex, Lex, Parse};
///
/// assert_eq!((171, "c"), hex().count(2).try_map(|s| u8::from_str_radix(s, 16)).parse("abc")?);
///
/// # Ok::<(), parsely::Error>(())
/// ```
///
/// Convert to `Vec<u8>`:
///
/// ```
/// use parsely::{hex, Lex, Parse};
///
/// let mut hex_bytes = hex().many(1..=2).try_map(|s| u8::from_str_radix(s, 16)).many(1..);
///
/// assert_eq!((vec![9, 10, 11, 12], ""), hex_bytes.parse("090A0B0C")?);
///
/// # Ok::<(), parsely::Error>(())
/// ```
///
/// # Note
///
//TODO: This lexer will not transform its output into another type, but this can be done using [`Lex::map`].
pub fn hex() -> Digit {
    Digit { radix: 16 }
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
            "digit() matches base 10 digits",
            digit(),
            &[
                ("abc", None, "abc"), //
                ("123", Some("1"), "23"),
                ("1.23", Some("1"), ".23"),
            ],
        );
    }
}
