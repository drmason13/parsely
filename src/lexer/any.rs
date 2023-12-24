use crate::{is_utf8_char_boundary, Lex};

/// This lexer is returned by [`any()`]. See it's documentation for more details.
#[derive(Debug, Clone)]
pub struct Any;

impl Lex for Any {
    fn lex<'i>(&self, input: &'i str) -> crate::LexResult<'i> {
        // shamelessly stolen from the implementation of unstable round_char_boundary feature
        if input.is_empty() {
            return Err(crate::Error::no_match(input));
        }

        // 1 UTF-8 char is at most 4 bytes in size
        let boundary = input.as_bytes()[..4]
            .iter()
            .position(|b| is_utf8_char_boundary(*b))
            .unwrap_or(4);

        Ok(input.split_at(boundary))
    }
}

/// This parser will match and consume 1 char of the input.
///
/// If the input is empty then it fails.
pub fn any() -> Any {
    Any
}
