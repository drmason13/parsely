use std::fmt;

use crate::{Lex, LexResult};

#[derive(Clone)]
pub struct Char(pub char);

impl Lex for Char {
    fn lex<'i>(&self, input: &'i str) -> LexResult<'i> {
        let mut chars = input.char_indices();

        match chars.next() {
            Some((_, c)) if c == self.0 => {
                let boundary = match chars.next() {
                    Some((j, _)) => j,
                    None => input.len(),
                };

                Ok(input.split_at(boundary))
            }
            _ => Err(crate::Error::NoMatch),
        }
    }
}

pub fn char(char: char) -> Char {
    Char(char)
}

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
                Err(crate::Error::NoMatch)
            }
        } else {
            Err(crate::Error::NoMatch)
        }
    }
}

pub fn char_if<F>(condition: F) -> CharIf<F>
where
    F: Fn(char) -> bool,
{
    CharIf { condition }
}

#[derive(Clone)]
pub struct WhiteSpace;

impl Lex for WhiteSpace {
    fn lex<'i>(&self, input: &'i str) -> LexResult<'i> {
        let mut chars = input.char_indices();

        match chars.next() {
            Some((_, c)) if c.is_whitespace() => {
                let boundary = match chars.next() {
                    Some((j, _)) => j,
                    None => input.len(),
                };

                Ok(input.split_at(boundary))
            }
            _ => Err(crate::Error::NoMatch),
        }
    }
}

pub fn ws() -> WhiteSpace {
    WhiteSpace
}

/// Matches an alphabetic character.
pub fn alpha() -> CharIf<fn(char) -> bool> {
    char_if(char::is_alphabetic)
}

/// Matches an alphanumeric character.
pub fn alphanum() -> CharIf<fn(char) -> bool> {
    char_if(char::is_alphanumeric)
}

/// Matches an ascii alphanumeric character.
pub fn ascii_alpha() -> CharIf<fn(char) -> bool> {
    char_if(|c| c.is_ascii_alphabetic())
}

/// Matches an ascii alphanumeric character.
pub fn ascii_alphanum() -> CharIf<fn(char) -> bool> {
    char_if(|c| c.is_ascii_alphanumeric())
}

pub fn lowercase() -> CharIf<fn(char) -> bool> {
    char_if(char::is_lowercase)
}

pub fn uppercase() -> CharIf<fn(char) -> bool> {
    char_if(char::is_uppercase)
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
            char('a'),
            &[
                ("ab", Some("a"), "b"), //
                ("abcd", Some("a"), "bcd"),
                ("zzz", None, "zzz"),
            ],
        );

        test_lexer_batch(
            "short input",
            char('a'),
            &[
                ("a", Some("a"), ""), //
                ("", None, ""),
                ("z", None, "z"),
            ],
        );

        test_lexer_batch(
            "unicode in lexer",
            char('â'),
            &[
                ("âb", Some("â"), "b"), //
                ("âbcd", Some("â"), "bcd"),
                ("zzz", None, "zzz"),
            ],
        );

        test_lexer_batch(
            "unicode in input",
            char('a'),
            &[
                ("aâb", Some("a"), "âb"), //
                ("aâbcd", Some("a"), "âbcd"),
                ("âââ", None, "âââ"),
            ],
        );

        test_lexer_batch(
            "unicode in lexer with short input",
            char('â'),
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
