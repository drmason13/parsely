use std::fmt;

use crate::{Parse, ParseError, ParseResult};

// This `struct` is created by the function `[token]`. See its documentation for more.
pub struct Token<'p>(pub &'p str);

impl<'p> Parse for Token<'p> {
    fn parse<'i>(&mut self, input: &'i str) -> ParseResult<'i> {
        if input.starts_with(self.0) {
            Ok(input.split_at(self.0.len()))
        } else {
            Err(ParseError::NoMatch)
        }
    }
}

/// A parser that matches a specific string slice.
///
/// This parser is useful for keywords or other specific sequences of characters in your input that should be matched.
///
/// Create this parser by providing the token to match.
///
/// When calling the [`Parse::parse`] method, this parser will return a tuple `(matched, remaining)` of the matched token and the remaining input.
///
//TODO: You can map this parser's output (which will be the matched token if successful) to another type using [`ParseResult::map`],
//TODO: and you can chain other parsers to parse the remaining input with [`ParseResult::then`].
///
/// # Examples
///
/// Basic usage:
///
/// ```
/// use parsely::{token, Parse, ParseError};
///
/// let input = "FOO 123";
///
/// let mut fooParser = token("FOO");
///
/// let (output, remaining) = fooParser.parse(input)?;
///
/// assert_eq!(output, "FOO");
/// assert_eq!(remaining, " 123");
///
/// # Ok::<(), ParseError>(())
/// ```
///
/// Map the output to a custom struct:
///
/// ```ignore
/// use parsely::{token, Parse, ParseError};
///
/// #[derive(Debug, PartialEq)]
/// struct Foo;
///
/// let input = "FOO 123";
///
/// let mut fooParser = token("FOO");
///
/// let (output, result) = fooParseer.parse(input).map(|_| Foo)?;
///
/// assert_eq!(output, Foo);
/// assert_eq!(result, " 123");
///
/// # Ok::<(), ParseError>(())
/// ```
pub fn token(token: &str) -> Token {
    Token(token)
}

impl fmt::Display for Token<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "token(\"{}\")", self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_utils::*;

    #[test]
    fn parsing() {
        test_parser_batch(
            "simple input",
            token("foo"),
            &[
                ("foob", Some("foo"), "b"), //
                ("foobcd", Some("foo"), "bcd"),
                ("zzz", None, "zzz"),
            ],
        );

        test_parser_batch(
            "short input",
            token("foo"),
            &[
                ("foo", Some("foo"), ""), //
                ("", None, ""),
                ("z", None, "z"),
            ],
        );

        test_parser_batch(
            "unicode in parser",
            token("Bâr"),
            &[
                ("Bârb", Some("Bâr"), "b"), //
                ("Bârbcd", Some("Bâr"), "bcd"),
                ("zzz", None, "zzz"),
            ],
        );

        test_parser_batch(
            "unicode in input",
            token("foo"),
            &[
                ("fooâb", Some("foo"), "âb"), //
                ("fooâbcd", Some("foo"), "âbcd"),
                ("âââ", None, "âââ"),
            ],
        );

        test_parser_batch(
            "unicode in parser with short input",
            token("Bâr"),
            &[
                ("Bâr", Some("Bâr"), ""), //
                ("", None, ""),
                ("z", None, "z"),
            ],
        );
    }

    #[test]
    fn token_parser_matches_char_parser() {
        test_parser_batch(
            "matches char: simple input",
            token("a"),
            &[
                ("ab", Some("a"), "b"), //
                ("abcd", Some("a"), "bcd"),
                // ("zzz", None, "zzz"),
            ],
        );

        test_parser_batch(
            "matches char: short input",
            token("a"),
            &[
                ("a", Some("a"), ""), //
                ("", None, ""),
                ("z", None, "z"),
            ],
        );

        test_parser_batch(
            "matches char: unicode in parser",
            token("â"),
            &[
                ("âb", Some("â"), "b"), //
                ("âbcd", Some("â"), "bcd"),
                ("zzz", None, "zzz"),
            ],
        );

        test_parser_batch(
            "matches char: unicode in input",
            token("a"),
            &[
                ("aâb", Some("a"), "âb"), //
                ("aâbcd", Some("a"), "âbcd"),
                ("âââ", None, "âââ"),
            ],
        );

        test_parser_batch(
            "matches char: unicode in parser with short input",
            token("â"),
            &[
                ("â", Some("â"), ""), //
                ("", None, ""),
                ("z", None, "z"),
            ],
        );
    }
}
