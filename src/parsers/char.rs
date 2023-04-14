use std::fmt;

use crate::{ParseResult, Parser};

pub struct Char(pub char);

impl Parser for Char {
    fn parse<'a>(&mut self, input: &'a str) -> ParseResult<'a> {
        let mut chars = input.char_indices();

        match chars.next() {
            Some((_, c)) if c == self.0 => {
                let boundary = match chars.next() {
                    Some((j, _)) => j,
                    None => input.len(),
                };

                let (output, remaining) = input.split_at(boundary);
                ParseResult::new(Some(output), remaining)
            }
            _ => ParseResult::new(None, input),
        }
    }
}

pub fn char(char: char) -> Char {
    Char(char)
}

impl fmt::Display for Char {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "char('{}')", self.0)
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
            char('a'),
            &[
                ("ab", Some("a"), "b"), //
                ("abcd", Some("a"), "bcd"),
                ("zzz", None, "zzz"),
            ],
        );

        test_parser_batch(
            "short input",
            char('a'),
            &[
                ("a", Some("a"), ""), //
                ("", None, ""),
                ("z", None, "z"),
            ],
        );

        test_parser_batch(
            "unicode in parser",
            char('â'),
            &[
                ("âb", Some("â"), "b"), //
                ("âbcd", Some("â"), "bcd"),
                ("zzz", None, "zzz"),
            ],
        );

        test_parser_batch(
            "unicode in input",
            char('a'),
            &[
                ("aâb", Some("a"), "âb"), //
                ("aâbcd", Some("a"), "âbcd"),
                ("âââ", None, "âââ"),
            ],
        );

        test_parser_batch(
            "unicode in parser with short input",
            char('â'),
            &[
                ("â", Some("â"), ""), //
                ("", None, ""),
                ("z", None, "z"),
            ],
        );
    }
}
