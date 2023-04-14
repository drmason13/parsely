use std::fmt;

use crate::{ParseResult, Parser};

// This `struct` is created by the function `[token]`. See its documentation for more.
pub struct Token<'p>(pub &'p str);

impl<'p> Parser for Token<'p> {
    fn parse<'a>(&mut self, input: &'a str) -> ParseResult<'a> {
        if input.starts_with(self.0) {
            let (processed, remaining) = input.split_at(self.0.len());
            ParseResult::new(Some(processed), remaining)
        } else {
            ParseResult::new(None, input)
        }
    }
}

/// A parser that matches a specific string slice.
///
/// This parser is useful for keywords or other specific sequences of characters in your input that should be matched.
///
/// Create this parser by providing the token to match.
///
/// When calling the [`Parser::parse`] method, this parser will return the matched token and the remaining input in a [`ParseResult`] struct.
///
/// You can map this parser's output (which will be the matched token if successful) to another type using [`ParseResult::map`],
/// and you can chain other parsers to parse the remaining input with [`ParseResult::then`].
///
/// # Examples
///
/// Basic usage:
///
/// ```
/// use parsely::{token, Parser};
///
/// let input = "FOO 123";
///
/// let mut fooParser = token("FOO");
///
/// let result = fooParser.parse(input);
///
/// assert_eq!(result.output(), Some("FOO"));
/// assert_eq!(result.remaining(), " 123");
/// ```
///
/// Map the output to a custom struct:
///
/// ```
/// use parsely::{token, Parser};
///
/// #[derive(Debug, PartialEq)]
/// struct Foo;
///
/// let input = "FOO 123";
///
/// let mut fooParser = token("FOO");
///
/// let (output, result) = fooParser.parse(input).map(|_| Foo);
///
/// assert_eq!(output, Some(Foo));
/// assert_eq!(result.remaining(), " 123");
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
                ("zzz", None, "zzz"),
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
