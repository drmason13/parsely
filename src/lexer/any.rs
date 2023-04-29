use crate::Lex;

/// This lexer is returned by [`any()`]. See it's documentation for more details.
#[derive(Debug, Clone)]
pub struct Any;

impl Lex for Any {
    fn lex<'i>(&self, input: &'i str) -> crate::LexResult<'i> {
        if let Some(c) = input.chars().next() {
            Ok(input.split_at(c.len_utf8()))
        } else {
            Err(crate::Error::NoMatch)
        }
    }
}

/// This parser will match and consume 1 char of the input.
///
/// If the input is empty then it fails.
pub fn any() -> Any {
    Any
}
