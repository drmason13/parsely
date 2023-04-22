use std::fmt;

use crate::{Lex, LexError, LexResult};

#[derive(Clone)]
pub struct Char(pub char);

impl Lex for Char {
    fn lex<'i>(&mut self, input: &'i str) -> LexResult<'i> {
        let mut chars = input.char_indices();

        match chars.next() {
            Some((_, c)) if c == self.0 => {
                let boundary = match chars.next() {
                    Some((j, _)) => j,
                    None => input.len(),
                };

                Ok(input.split_at(boundary))
            }
            _ => Err(LexError::NoMatch),
        }
    }
}

pub fn char(char: char) -> Char {
    Char(char)
}

#[derive(Clone)]
pub struct WhiteSpace;

impl Lex for WhiteSpace {
    fn lex<'i>(&mut self, input: &'i str) -> LexResult<'i> {
        let mut chars = input.char_indices();

        match chars.next() {
            Some((_, c)) if c.is_whitespace() => {
                let boundary = match chars.next() {
                    Some((j, _)) => j,
                    None => input.len(),
                };

                Ok(input.split_at(boundary))
            }
            _ => Err(LexError::NoMatch),
        }
    }
}

pub fn ws() -> WhiteSpace {
    WhiteSpace
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
