use std::marker::PhantomData;

use crate::{Behavior, InProgressError, Lex, Lexing, Parse, Parsing};

/// This parser/lexer is returned by [`escape()`]/[`escape_lex()`], see their documentation for more details
pub struct EscapeSequence<const N: usize, B: Behavior> {
    escape_char: char,
    sequences: [(char, char); N],
    behavior: PhantomData<B>,
}

impl<const N: usize, B: Behavior> EscapeSequence<N, B> {
    /// Switches the behavior of this combinator to [`Lexing`]
    pub fn lexing(self) -> EscapeSequence<N, Lexing> {
        EscapeSequence {
            escape_char: self.escape_char,
            sequences: self.sequences,
            behavior: PhantomData::<Lexing>,
        }
    }

    /// Switches the behavior of this combinator to [`Parsing`]
    pub fn parsing(self) -> EscapeSequence<N, Parsing> {
        EscapeSequence {
            escape_char: self.escape_char,
            sequences: self.sequences,
            behavior: PhantomData::<Parsing>,
        }
    }
}

impl<const N: usize> Parse for EscapeSequence<N, Parsing> {
    type Output = char;

    fn parse<'i>(&self, input: &'i str) -> crate::ParseResult<'i, Self::Output> {
        let mut chars = input.chars();
        let next_char = chars
            .next()
            .ok_or_else(|| InProgressError::no_match(input))?;

        if next_char == self.escape_char {
            let char_after_next = chars
                .next()
                .ok_or_else(|| InProgressError::no_match(input))?;

            for (escaped_char, output) in self.sequences.iter() {
                if char_after_next == *escaped_char {
                    let remaining = input
                        .split_at(next_char.len_utf8() + char_after_next.len_utf8())
                        .1;
                    return Ok((*output, remaining));
                }
            }
            // invalid escape sequence
            Err(InProgressError::failed_conversion(input))
        } else {
            Ok((next_char, input.split_at(next_char.len_utf8()).1))
        }
    }
}

impl<const N: usize> Lex for EscapeSequence<N, Lexing> {
    fn lex<'i>(&self, input: &'i str) -> crate::LexResult<'i> {
        let mut chars = input.chars();
        let next_char = chars
            .next()
            .ok_or_else(|| InProgressError::no_match(input))?;

        if next_char == self.escape_char {
            let char_after_next = chars
                .next()
                .ok_or_else(|| InProgressError::no_match(input))?;

            for (escaped_char, _) in self.sequences.iter() {
                if char_after_next == *escaped_char {
                    return Ok(input.split_at(next_char.len_utf8() + char_after_next.len_utf8()));
                }
            }
            // invalid escape sequence
            Err(InProgressError::failed_conversion(input))
        } else {
            Ok(input.split_at(next_char.len_utf8()))
        }
    }
}

/// Returns a [parser](Parse) that transforms escape sequences into one character, or returns the non-escpape char unchanged
///
/// See also [`escape_lex()`] to lex escape sequences without transforming the input
///
/// # Examples
///
/// Basic usage:
///
/// ```
/// use parsely::Parse;
///
/// pub fn escape() -> impl Parse<Output = char> {
///     // \ is the escape character, that starts an escape sequence
///     parsely::escape('\\', [
///         // map \n to a newline character
///         ('n', '\n'),
///         // map \t to a tab character
///         ('t', '\t'),
///         // map \\ to \
///         ('\\', '\\'),
///     ])
/// }
///
/// // escape sequences are escaped
/// assert_eq!(escape().parse(r"\n")?, ('\n', ""));
/// assert_eq!(escape().parse(r"\\n")?, ('\\', "n"));
///
/// // non escape characters are unchanged
/// assert_eq!(escape().parse("abc")?, ('a', "bc"));
/// #
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
pub fn escape<const N: usize>(
    escape_char: char,
    sequences: [(char, char); N],
) -> EscapeSequence<N, Parsing> {
    EscapeSequence {
        escape_char,
        sequences,
        behavior: PhantomData::<Parsing>,
    }
}

/// Returns a [lexer](Lex) that matches escape sequences
///
/// See also [`escape()`] to parse escape sequences and output the escaped character
///
/// # Examples
///
/// Basic usage:
///
/// ```
/// use parsely::Lex;
///
/// pub fn escape_lex() -> impl Lex {
///     // \ is the escape character, that starts an escape sequence
///     parsely::escape_lex('\\', [
///         // map \n to a newline character
///         ('n', '\n'),
///         // map \t to a tab character
///         ('t', '\t'),
///         // map \\ to \
///         ('\\', '\\'),
///     ])
/// }
///
/// // escape sequences match, but are not escaped
/// assert_eq!(escape_lex().lex(r"\n")?, (r"\n", ""));
/// assert_eq!(escape_lex().lex(r"\\n")?, (r"\\", "n"));
///
/// // non escape characters are of course also unchanged
/// assert_eq!(escape_lex().lex("abc")?, ("a", "bc"));
/// #
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
pub fn escape_lex<const N: usize>(
    escape_char: char,
    sequences: [(char, char); N],
) -> EscapeSequence<N, Lexing> {
    EscapeSequence {
        escape_char,
        sequences,
        behavior: PhantomData::<Lexing>,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{ErrorReason, Lex, Parse};

    #[test]
    fn escape_sequence_works() -> Result<(), Box<dyn std::error::Error>> {
        let test = escape('\\', [('n', '\n'), ('r', '\r'), ('t', '\t'), ('"', '"')]);

        assert_eq!(test.parse("a")?, ('a', ""));
        assert_eq!(test.parse("\\t")?, ('\t', ""));

        Ok(())
    }

    #[test]
    fn escape_sequence_errors() -> Result<(), Box<dyn std::error::Error>> {
        let test = escape('\\', [('n', '\n'), ('r', '\r'), ('t', '\t'), ('"', '"')]);

        assert_eq!(test.parse("\\a").unwrap_err().remaining, "\\a");

        Ok(())
    }

    #[test]
    fn escape_sequence_string_works() -> Result<(), Box<dyn std::error::Error>> {
        let escape_sequence = escape('\\', [('n', '\n'), ('r', '\r'), ('t', '\t'), ('"', '"')]);

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
        let escape_sequence = escape('\\', [('n', '\n'), ('r', '\r'), ('t', '\t'), ('"', '"')]);

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
        let escape_sequence = escape('\\', [('n', '\n'), ('r', '\r'), ('t', '\t'), ('"', '"')]);

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
