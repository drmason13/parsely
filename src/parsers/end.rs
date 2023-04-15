use crate::{ParseResult, Parser};

pub struct End;

impl Parser for End {
    fn parse<'a>(&mut self, input: &'a str) -> crate::ParseResult<'a> {
        if input.is_empty() {
            ParseResult::new(Some(""), "")
        } else {
            ParseResult::new(None, input)
        }
    }
}

pub fn end() -> End {
    End
}
