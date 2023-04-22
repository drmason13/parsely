use crate::Lex;

#[derive(Debug, Clone)]
pub struct Any;

impl Lex for Any {
    fn lex<'i>(&mut self, input: &'i str) -> crate::LexResult<'i> {
        if let Some(c) = input.chars().next() {
            Ok(input.split_at(c.len_utf8()))
        } else {
            Err(crate::Error::NoMatch)
        }
    }
}

/// This parser will match and consume 1 char of the input.
///
/// If the input is empty it fails.
pub fn any() -> Any {
    Any
}
