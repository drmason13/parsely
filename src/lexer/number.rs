use std::fmt;

use crate::{char_if, Lex, LexResult};

/// This lexer is returned by [`digit()`]. See it's documentation for more details.
#[derive(Clone)]
pub struct Digit {
    radix: u32,
}

impl Lex for Digit {
    fn lex<'i>(&self, input: &'i str) -> LexResult<'i> {
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

/// This lexer matches a single base 10 digit i.e. one of "1234567890".
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

/// This lexer matches a single non-zero base 10 digit i.e. one of "123456789".
pub fn non_zero_digit() -> impl Lex + Clone {
    char_if(|c| c.is_ascii_digit() && c != '0')
}

/// This lexer matches a single hexadecimal character, i.e. one of "0123456789abcdefABCDEF".
///
/// No leading `0x` or other hex notation in the input is accepted.
///
/// As this is a lexer, no type conversion is performed, see the examples for how you might want to do this.
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
/// assert_eq!(("0123456789abcdef", "g"), hex().many(1..).lex("0123456789abcdefg")?);
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
/// let hex_bytes = hex().many(1..=2).try_map(|s| u8::from_str_radix(s, 16)).many(1..);
///
/// assert_eq!((vec![9, 10, 11, 12, 7], ""), hex_bytes.parse("090A0B0C7")?);
///
/// # Ok::<(), parsely::Error>(())
/// ```
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
