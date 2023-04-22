use std::fmt;

use crate::{Lex, LexError, LexResult};

// This `struct` is created by the function `[token]`. See its documentation for more.
pub struct Token<'p>(pub &'p str);

impl<'p> Lex for Token<'p> {
    fn lex<'i>(&mut self, input: &'i str) -> LexResult<'i> {
        if input.starts_with(self.0) {
            Ok(input.split_at(self.0.len()))
        } else {
            Err(LexError::NoMatch)
        }
    }
}

/// A lexer that matches a specific string slice.
///
/// This lexer is useful for keywords or other specific sequences of characters in your input that should be matched.
///
/// Create this lexer by providing the token to match.
///
/// When calling the [`Lex::lex`] method, this lexer will return a tuple `(matched, remaining)` of the matched token and the remaining input.
///
//TODO: You can map this lexer's output (which will be the matched token if successful) to another type using [`LexResult::map`],
//TODO: and you can chain other lexers to lex the remaining input with [`LexResult::then`].
///
/// # Examples
///
/// Basic usage:
///
/// ```
/// use parsely::{token, Lex, LexError};
///
/// let input = "FOO 123";
///
/// let mut fooLexr = token("FOO");
///
/// let (output, remaining) = fooLexr.lex(input)?;
///
/// assert_eq!(output, "FOO");
/// assert_eq!(remaining, " 123");
///
/// # Ok::<(), LexError>(())
/// ```
///
/// Map the output to a custom struct:
///
/// ```ignore
/// use parsely::{token, Lex, LexError};
///
/// #[derive(Debug, PartialEq)]
/// struct Foo;
///
/// let input = "FOO 123";
///
/// let mut fooLexr = token("FOO");
///
/// let (output, result) = fooLexer.lex(input).map(|_| Foo)?;
///
/// assert_eq!(output, Foo);
/// assert_eq!(result, " 123");
///
/// # Ok::<(), LexError>(())
/// ```
pub fn token(token: &str) -> Token {
    Token(token)
}

impl fmt::Debug for Token<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Token(\"{}\")", self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_utils::*;

    #[test]
    fn parsing() {
        test_lexer_batch(
            "simple input",
            token("foo"),
            &[
                ("foob", Some("foo"), "b"), //
                ("foobcd", Some("foo"), "bcd"),
                ("zzz", None, "zzz"),
            ],
        );

        test_lexer_batch(
            "short input",
            token("foo"),
            &[
                ("foo", Some("foo"), ""), //
                ("", None, ""),
                ("z", None, "z"),
            ],
        );

        test_lexer_batch(
            "unicode in lexer",
            token("Bâr"),
            &[
                ("Bârb", Some("Bâr"), "b"), //
                ("Bârbcd", Some("Bâr"), "bcd"),
                ("zzz", None, "zzz"),
            ],
        );

        test_lexer_batch(
            "unicode in input",
            token("foo"),
            &[
                ("fooâb", Some("foo"), "âb"), //
                ("fooâbcd", Some("foo"), "âbcd"),
                ("âââ", None, "âââ"),
            ],
        );

        test_lexer_batch(
            "unicode in lexer with short input",
            token("Bâr"),
            &[
                ("Bâr", Some("Bâr"), ""), //
                ("", None, ""),
                ("z", None, "z"),
            ],
        );
    }

    #[test]
    fn token_lexer_matches_char_lexer() {
        test_lexer_batch(
            "matches char: simple input",
            token("a"),
            &[
                ("ab", Some("a"), "b"), //
                ("abcd", Some("a"), "bcd"),
                // ("zzz", None, "zzz"),
            ],
        );

        test_lexer_batch(
            "matches char: short input",
            token("a"),
            &[
                ("a", Some("a"), ""), //
                ("", None, ""),
                ("z", None, "z"),
            ],
        );

        test_lexer_batch(
            "matches char: unicode in lexer",
            token("â"),
            &[
                ("âb", Some("â"), "b"), //
                ("âbcd", Some("â"), "bcd"),
                ("zzz", None, "zzz"),
            ],
        );

        test_lexer_batch(
            "matches char: unicode in input",
            token("a"),
            &[
                ("aâb", Some("a"), "âb"), //
                ("aâbcd", Some("a"), "âbcd"),
                ("âââ", None, "âââ"),
            ],
        );

        test_lexer_batch(
            "matches char: unicode in lexer with short input",
            token("â"),
            &[
                ("â", Some("â"), ""), //
                ("", None, ""),
                ("z", None, "z"),
            ],
        );
    }
}
