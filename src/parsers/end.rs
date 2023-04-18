use crate::{Parse, ParseError, ParseResult};

pub struct End;

impl Parse for End {
    fn parse<'i>(&mut self, input: &'i str) -> ParseResult<'i> {
        if input.is_empty() {
            Ok(("", ""))
        } else {
            Err(ParseError::NoMatch)
        }
    }
}

pub fn end() -> End {
    End
}
