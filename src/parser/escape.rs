use crate::{Error, Parse};

pub struct EscapeSequence<const N: usize> {
    escape_char: char,
    sequences: [(char, char); N],
}

impl<const N: usize> Parse for EscapeSequence<N> {
    type Output = char;

    fn parse<'i>(&self, input: &'i str) -> crate::ParseResult<'i, Self::Output> {
        let mut chars = input.chars();
        let next_char = chars.next().ok_or_else(|| Error::no_match(input))?;

        if next_char == self.escape_char {
            let char_after_next = chars.next().ok_or_else(|| Error::no_match(input))?;

            for (escaped_char, output) in self.sequences.iter() {
                if char_after_next == *escaped_char {
                    let remaining = input
                        .split_at(next_char.len_utf8() + char_after_next.len_utf8())
                        .1;
                    return Ok((*output, remaining));
                }
            }
            // invalid escape sequence
            Err(Error::failed_conversion(input))
        } else {
            Ok((next_char, input.split_at(next_char.len_utf8()).1))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{ErrorReason, Lex, Parse};

    #[test]
    fn escape_sequence_works() -> Result<(), Box<dyn std::error::Error>> {
        let test = EscapeSequence {
            escape_char: '\\',
            sequences: [('n', '\n'), ('r', '\r'), ('t', '\t'), ('"', '"')],
        };

        assert_eq!(test.parse("a")?, ('a', ""));
        assert_eq!(test.parse("\\t")?, ('\t', ""));

        Ok(())
    }

    #[test]
    fn escape_sequence_errors() -> Result<(), Box<dyn std::error::Error>> {
        let test = EscapeSequence {
            escape_char: '\\',
            sequences: [('n', '\n'), ('r', '\r'), ('t', '\t'), ('"', '"')],
        };

        assert_eq!(test.parse("\\a").unwrap_err().remaining, "\\a");

        Ok(())
    }

    #[test]
    fn escape_sequence_string_works() -> Result<(), Box<dyn std::error::Error>> {
        let escape_sequence = EscapeSequence {
            escape_char: '\\',
            sequences: [('n', '\n'), ('r', '\r'), ('t', '\t'), ('"', '"')],
        };

        let test = '"'
            .skip_then(escape_sequence.many(..).or_until('"').collect::<String>())
            .then_skip('"');

        assert_eq!(test.parse(r#""abc""#)?, ("abc".to_string(), ""));
        assert_eq!(
            test.parse(r#""abc\n\t123\r\n""#)?,
            ("abc\n\t123\r\n".to_string(), "")
        );

        Ok(())
    }

    #[test]
    fn escape_sequence_string_escapes_quotes() -> Result<(), Box<dyn std::error::Error>> {
        let escape_sequence = EscapeSequence {
            escape_char: '\\',
            sequences: [('n', '\n'), ('r', '\r'), ('t', '\t'), ('"', '"')],
        };

        let test = '"'
            .skip_then(escape_sequence.many(..).or_until('"').collect::<String>())
            .then_skip('"');

        assert_eq!(
            test.parse(r#""abc\n123: \"string in string\"""#)?,
            ("abc\n123: \"string in string\"".to_string(), "")
        );

        Ok(())
    }

    #[test]
    fn escape_sequence_string_errors() -> Result<(), Box<dyn std::error::Error>> {
        let escape_sequence = EscapeSequence {
            escape_char: '\\',
            sequences: [('n', '\n'), ('r', '\r'), ('t', '\t'), ('"', '"')],
        };

        let test = '"'
            .skip_then(escape_sequence.many(..).or_until('"').collect::<String>())
            .then_skip('"');

        // invalid escape sequence
        let err = test.parse(r#""abc\a123""#).unwrap_err();
        assert_eq!(err.remaining, "\\a123\"");
        // TODO: create a sequence combinator that preserves errors encountered during parsing, so that we get ErrorReason::FailedConversion here
        assert_eq!(err.reason, ErrorReason::NoMatch);

        // missing closing quote
        let err = test.parse(r#""abc\n123"#).unwrap_err();
        assert_eq!(err.remaining, "");
        assert_eq!(err.reason, ErrorReason::NoMatch);

        Ok(())
    }
}
