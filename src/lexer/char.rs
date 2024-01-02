use std::fmt;

use crate::{Lex, LexResult};

/// This lexer is returned by [`char()`]. See it's documentation for more details.
#[derive(Clone)]
pub struct Char(pub char);

impl Lex for Char {
    fn lex<'i>(&self, input: &'i str) -> LexResult<'i> {
        let mut chars = input.char_indices();

        match chars.next() {
            Some((_, c)) if c == self.0 => {
                let boundary = match chars.next() {
                    Some((n, _)) => n,
                    None => input.len(),
                };

                Ok(input.split_at(boundary))
            }
            _ => Err(crate::Error::no_match(input)),
        }
    }
}

/// This lexer matches the given [`char`](prim@char) once.
///
/// See also [`token()`](crate::token) to match strings rather than single chars.
///
/// # Examples
///
/// ```
/// use parsely::{ch, Lex};
///
/// let matches_a = ch('a');
///
/// let (output, remaining) = matches_a.lex("abc")?;
/// assert_eq!(output, "a");
/// assert_eq!(remaining, "bc");
/// # Ok::<(), parsely::Error>(())
/// ```
///
/// Note that using the [`char`](prim@char) primitive is equivalent to wrapping that char in [`char()`].
///
/// the above example can be shortened to:
///
/// ```
/// use parsely::{Lex};
/// // Note: the Lex trait must be in scope for this to work
///
/// let (output, remaining) = 'a'.lex("abc")?;
/// assert_eq!(output, "a");
/// assert_eq!(remaining, "bc");
/// # Ok::<(), parsely::Error>(())
/// ```
///
/// Using chars directly is preferred in the examples throughout this documentation.
pub fn ch(ch: char) -> Char {
    Char(ch)
}

/// This lexer is returned by [`ch_if()`]. See it's documentation for more details.
#[derive(Clone)]
pub struct CharIf<F> {
    condition: F,
}

impl<F> Lex for CharIf<F>
where
    F: Fn(char) -> bool,
{
    fn lex<'i>(&self, input: &'i str) -> LexResult<'i> {
        if let Some(c) = input.chars().next() {
            if (self.condition)(c) {
                Ok(input.split_at(c.len_utf8()))
            } else {
                Err(crate::Error::no_match(input))
            }
        } else {
            Err(crate::Error::no_match(input))
        }
    }
}

/// This lexer matches a single [`char`](prim@char) if it satisfies the given condition.
///
/// # Examples
///
/// ```
/// use parsely::{ch_if, Lex};
///
/// let uppercase = ch_if(|c| c.is_ascii_uppercase());
///
/// let (output, remaining) = uppercase.lex("ABC")?;
/// assert_eq!(output, "A");
/// assert_eq!(remaining, "BC");
/// # Ok::<(), parsely::Error>(())
/// ```
pub fn ch_if<F>(condition: F) -> CharIf<F>
where
    F: Fn(char) -> bool,
{
    CharIf { condition }
}

/// This lexer is returned by [`ws()`]. See it's documentation for more details.
#[derive(Clone)]
pub struct WhiteSpace;

impl Lex for WhiteSpace {
    fn lex<'i>(&self, input: &'i str) -> LexResult<'i> {
        let mut chars = input.char_indices();

        match chars.next() {
            Some((_, c)) if c.is_whitespace() => {
                let boundary = match chars.next() {
                    Some((n, _)) => n,
                    None => input.len(),
                };

                Ok(input.split_at(boundary))
            }
            _ => Err(crate::Error::no_match(input)),
        }
    }
}

/// This lexer matches a single [`char`](prim@char) if it is a whitespace character.
pub fn ws() -> WhiteSpace {
    WhiteSpace
}

/// Matches a single alphabetic character.
pub fn alpha() -> CharIf<fn(char) -> bool> {
    ch_if(char::is_alphabetic)
}

/// Matches a single alphanumeric character.
pub fn alphanum() -> CharIf<fn(char) -> bool> {
    ch_if(char::is_alphanumeric)
}

/// Matches a single ascii alphanumeric character.
pub fn ascii_alpha() -> CharIf<fn(char) -> bool> {
    ch_if(|c| c.is_ascii_alphabetic())
}

/// Matches a single ascii alphanumeric character.
pub fn ascii_alphanum() -> CharIf<fn(char) -> bool> {
    ch_if(|c| c.is_ascii_alphanumeric())
}

/// Matches a single lowercase character.
pub fn lowercase() -> CharIf<fn(char) -> bool> {
    ch_if(char::is_lowercase)
}

/// Matches an uppercase character.
pub fn uppercase() -> CharIf<fn(char) -> bool> {
    ch_if(char::is_uppercase)
}

/// Matches a char that is one of the characters in the given string
///
/// # Examples
///
/// ```
/// use parsely::{one_of, Lex};
///
/// let (matched, remaining) = one_of("abc").lex("char")?;
/// assert_eq!(matched, "c");
/// assert_eq!(remaining, "har");
///
/// let result = one_of("abc").lex("har");
/// assert!(result.is_err());
///
/// # Ok::<(), parsely::Error>(())
/// ```
pub fn one_of(chars: &str) -> impl Lex + '_ {
    ch_if(|c| chars.contains(c))
}

/// Matches a char that is *none* of the characters in the given string.
///
/// # Examples
///
/// ```
/// use parsely::{none_of, Lex};
///
/// let result = none_of("abc").lex("char");
/// assert!(result.is_err());
///
/// let (matched, remaining) = none_of("abc").lex("har")?;
/// assert_eq!(matched, "h");
/// assert_eq!(remaining, "ar");
///
/// # Ok::<(), parsely::Error>(())
/// ```
pub fn none_of(chars: &str) -> impl Lex + '_ {
    ch_if(|c| !chars.contains(c))
}

impl fmt::Debug for Char {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Char('{}')", self.0)
    }
}

impl fmt::Debug for WhiteSpace {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "WhiteSpace")
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
            ch('a'),
            &[
                ("ab", Some("a"), "b"), //
                ("abcd", Some("a"), "bcd"),
                ("zzz", None, "zzz"),
            ],
        );

        test_lexer_batch(
            "short input",
            ch('a'),
            &[
                ("a", Some("a"), ""), //
                ("", None, ""),
                ("z", None, "z"),
            ],
        );

        test_lexer_batch(
            "unicode in lexer",
            ch('â'),
            &[
                ("âb", Some("â"), "b"), //
                ("âbcd", Some("â"), "bcd"),
                ("zzz", None, "zzz"),
            ],
        );

        test_lexer_batch(
            "unicode in input",
            ch('a'),
            &[
                ("aâb", Some("a"), "âb"), //
                ("aâbcd", Some("a"), "âbcd"),
                ("âââ", None, "âââ"),
            ],
        );

        test_lexer_batch(
            "unicode in lexer with short input",
            ch('â'),
            &[
                ("â", Some("â"), ""), //
                ("", None, ""),
                ("z", None, "z"),
            ],
        );

        test_lexer_batch(
            "whitespace",
            ws(),
            &[
                (" ", Some(" "), ""), //
                ("\t\r\n", Some("\t"), "\r\n"),
                ("\r\n", Some("\r"), "\n"),
                ("\n\r\t", Some("\n"), "\r\t"),
                ("z", None, "z"),
                (" \tâ", Some(" "), "\tâ"),
            ],
        );
    }
}
