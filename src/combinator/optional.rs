use crate::{Lex, Parse, ParseResult};

pub struct Optional<T> {
    item: T,
}

impl<T> Lex for Optional<T>
where
    T: Lex,
{
    fn lex<'i>(&mut self, input: &'i str) -> crate::LexResult<'i> {
        if let Ok((matched, remaining)) = self.item.lex(input) {
            Ok((matched, remaining))
        } else {
            Ok(("", input))
        }
    }
}

impl<T> Parse for Optional<T>
where
    T: Parse,
{
    type Output = Option<<T as Parse>::Output>;

    fn parse<'i>(&mut self, input: &'i str) -> ParseResult<'i, Self::Output> {
        if let Ok((output, remaining)) = self.item.parse(input) {
            Ok((Some(output), remaining))
        } else {
            Ok((None, input))
        }
    }
}

/// Makes an optional parser/lexer.
///
/// If the match fails, then the error is silenced and the input is returned as remaining input.
pub fn optional<T>(item: T) -> Optional<T> {
    Optional { item }
}
