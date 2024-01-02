use std::{borrow::Cow, fmt, marker::PhantomData};

use crate::{Lex, LexResult};

/// This lexer is returned by [`token()`]. See its documentation for more details.
#[derive(Clone)]
pub struct Token<'p, C: CaseSensitivity>(Cow<'p, str>, PhantomData<C>);

pub trait CaseSensitivity {}

pub struct CaseSensitive;
pub struct CaseInsensitive;

impl CaseSensitivity for CaseSensitive {}
impl CaseSensitivity for CaseInsensitive {}

impl<'p> Token<'p, CaseSensitive> {
    /// Makes the token case insensitive, that is the case of input characters is ignored.
    ///
    /// # Examples
    ///
    /// ```
    /// use parsely::{token, Lex};
    ///
    /// let case_sensitive = token("abc");
    /// let case_insensitive = token("abc").any_case();
    ///
    /// assert!(case_sensitive.lex("aBc").is_err());
    ///
    /// assert_eq!(case_insensitive.lex("aBc")?, ("aBc", ""));
    /// # Ok::<(), parsely::Error>(())
    /// ```
    pub fn any_case(self) -> Token<'p, CaseInsensitive> {
        Token(Cow::Owned(self.0.to_uppercase()), PhantomData)
    }
}

impl<'p> Lex for Token<'p, CaseSensitive> {
    fn lex<'i>(&self, input: &'i str) -> LexResult<'i> {
        if input.starts_with(self.0.as_ref()) {
            Ok(input.split_at(self.0.len()))
        } else {
            Err(crate::Error::no_match(input))
        }
    }
}

impl<'p> Lex for Token<'p, CaseInsensitive> {
    fn lex<'i>(&self, input: &'i str) -> LexResult<'i> {
        // NOTE: unicode uppercase could wreak havoc here
        if input.to_uppercase().starts_with(self.0.as_ref()) {
            Ok(input.split_at(self.0.len()))
        } else {
            Err(crate::Error::no_match(input))
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
/// # Examples
///
/// ```
/// use parsely::{token, Lex};
///
/// let (output, remaining) = token("FOO").lex("FOO 123")?;
///
/// assert_eq!(output, "FOO");
/// assert_eq!(remaining, " 123");
///
/// # Ok::<(), parsely::Error>(())
/// ```
///
/// [`&'static str`](prim@str) literals impl [`Lex`] directly by wraping them in `token()`.
///
/// This means the above example can be shortened:
///
/// ```
/// // the Lex trait must be in scope for this to work
/// use parsely::Lex;
///
/// let (output, remaining) = "FOO".lex("FOO 123")?;
///
/// assert_eq!(output, "FOO");
/// assert_eq!(remaining, " 123");
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
///
/// Using strings directly is preferred in the examples throughout this documentation.
///
/// Map the output of a lexer to create a parser:
///
/// Consider also using [`switch()`](crate::switch) if you have lots of tokens to map to cheap clone/copy values
///
/// ```
/// use parsely::{Lex, Parse};
///
/// #[derive(Debug, PartialEq)]
/// struct Foo;
///
/// let (output, result) = "FOO".map(|_| Foo).parse("FOO 123")?;
///
/// assert_eq!(output, Foo);
/// assert_eq!(result, " 123");
///
/// # Ok::<(), parsely::Error>(())
/// ```
pub fn token(token: &str) -> Token<CaseSensitive> {
    Token(Cow::Borrowed(token), PhantomData)
}

/// case Insensitive version of [`token`].
///
/// The token is converted to uppercase when creating the lexer. The input is uppercased before checking if the token matches every time the lexer runs.
/// This unsurprisingly incurs a performance penalty.
///
/// Note: no additional action is taken to support all unicode characters,
/// it is quite likely that this uppercase comparison will lead to unintuitive results for some unicode characters. Caution advised.
pub fn itoken(token: &str) -> Token<CaseInsensitive> {
    Token(Cow::Owned(token.to_uppercase()), PhantomData)
}

impl fmt::Debug for Token<'_, CaseSensitive> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Token(\"{}\")", self.0)
    }
}

impl fmt::Debug for Token<'_, CaseInsensitive> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Token(\"{}\", i)", self.0)
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
