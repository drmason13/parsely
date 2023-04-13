//!
//! Parsely is a parser combinator library for Rust with the following aims
//!
//! * Simple to use
//! * Well documented built-in parsers
//!
//! While it doesn't prioritise speed, it will often be "fast enough" for a projects that do a little bit of parsing here and there.
//!
//! If parsing speed is important to your application's performance (for example a compiler) then this library isn't meant for you.

#[derive(Debug, PartialEq, Eq)]
pub struct ParseOutput<'a> {
    processed: Option<&'a str>,
    remaining: &'a str,
}

impl<'a> ParseOutput<'a> {
    pub fn new(processed: Option<&'a str>, remaining: &'a str) -> Self {
        ParseOutput {
            processed,
            remaining,
        }
    }

    pub fn and(self, mut parser: impl Parser) -> Self {
        match self.processed {
            Some(next) => parser.parse(next),
            None => self,
        }
    }

    pub fn pipe(self, mut parser: impl Parser) -> Self {
        parser.parse_piped(self)
    }

    pub fn map<F, O>(self, f: F) -> Option<O>
    where
        F: FnMut(&'a str) -> O,
    {
        self.processed.map(f)
    }
}

pub trait Parser {
    fn parse<'a>(&mut self, input: &'a str) -> ParseOutput<'a>;

    fn parse_piped<'a>(&mut self, input: ParseOutput<'a>) -> ParseOutput<'a> {
        let input = input.remaining;
        self.parse(input)
    }
}

pub struct Token(char);

impl Parser for Token {
    fn parse<'a>(&mut self, input: &'a str) -> ParseOutput<'a> {
        let mut chars = input.char_indices();

        match chars.next() {
            Some((_, c)) if c == self.0 => {
                let boundary = match chars.next() {
                    Some((j, _)) => j,
                    None => input.len(),
                };

                let (processed, remaining) = input.split_at(boundary);
                ParseOutput::new(Some(processed), remaining)
            }
            _ => ParseOutput::new(None, input),
        }
    }
}

pub trait Head: Sized {
    /// Splits this type into 2 parts, the first "chunk" and the rest
    ///
    /// What a "chunk" is depends on the type implementing Head, for &str each chunk contains one character
    fn head(self) -> (Self, Self);

    fn head_if<F>(self, pred: F) -> (Self, Self)
    where
        F: Fn(char) -> bool;
}

impl Head for &str {
    fn head(self) -> (Self, Self) {
        head_str(self)
    }

    fn head_if<F>(self, pred: F) -> (Self, Self)
    where
        F: Fn(char) -> bool,
    {
        head_str_if(self, pred)
    }
}

/// Useful function to split a str into 2: its first character (head) and the rest (tail)
pub fn head_str(input: &str) -> (&str, &str) {
    let mut chars = input.char_indices();

    match chars.next() {
        Some(_) => {
            let boundary = match chars.next() {
                Some((j, _)) => j,
                None => input.len(),
            };

            input.split_at(boundary)
        }
        None => ("", ""),
    }
}

/// Useful function to split a str into 2: its first character (head) and the rest (tail)
pub fn head_str_if<F>(input: &str, pred: F) -> (&str, &str)
where
    F: Fn(char) -> bool,
{
    let mut chars = input.char_indices();

    match chars.next() {
        Some((_, c)) if pred(c) => {
            let boundary = match chars.next() {
                Some((j, _)) => j,
                None => input.len(),
            };

            input.split_at(boundary)
        }
        _ => ("", input),
    }
}

pub fn token(token: char) -> Token {
    Token(token)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_tokens() {
        test_parser(token('a'), "ab", Some("a"), "b");
        test_parser(token('a'), "abcd", Some("a"), "bcd");
        test_parser(token('a'), "zzz", None, "zzz");
    }

    #[test]
    fn test_parse_simple_tokens_short_input() {
        test_parser(token('a'), "a", Some("a"), "");
        test_parser(token('a'), "", None, "");
        test_parser(token('a'), "z", None, "z");
    }

    #[test]
    fn test_parse_unicode_tokens() {
        test_parser(token('â'), "âb", Some("â"), "b");
        test_parser(token('â'), "âbcd", Some("â"), "bcd");
        test_parser(token('â'), "zzz", None, "zzz");

        test_parser(token('a'), "aâb", Some("a"), "âb");
        test_parser(token('a'), "aâbcd", Some("a"), "âbcd");
        test_parser(token('a'), "âââ", None, "âââ");
    }

    #[test]
    fn test_parse_unicode_tokens_short_input() {
        test_parser(token('â'), "â", Some("â"), "");
        test_parser(token('â'), "", None, "");
        test_parser(token('â'), "z", None, "z");

        test_parser(token('a'), "â", None, "â");
        test_parser(token('a'), "", None, "");
        test_parser(token('a'), "âz", None, "âz");
    }

    fn test_parser(
        mut parser: impl Parser,
        input: &str,
        expected_output: Option<&str>,
        expected_remaining: &str,
    ) {
        assert_eq!(
            parser.parse(input),
            ParseOutput::new(expected_output, expected_remaining)
        );
    }
}
