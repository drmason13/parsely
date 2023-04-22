use crate::{Lex, Parse};

#[derive(Debug, Clone)]
pub struct Skip<L, T> {
    lexer: L,
    item: T,
}

/// After running the item (parser or lexer), this combinator will run the lexer and discard its output.
///
/// If the lexer fails, it is still a parse failure. Use `.optional()` if the input to be skipped isn't required.
pub fn skip<L, T>(lexer: L, item: T) -> Skip<L, T> {
    Skip { lexer, item }
}

impl<L: Lex, T: Lex> Lex for Skip<L, T> {
    fn lex<'i>(&mut self, input: &'i str) -> crate::LexResult<'i> {
        let (output, remaining) = self.item.lex(input)?;
        let (_, remaining) = self.lexer.lex(remaining)?;
        Ok((output, remaining))
    }
}

impl<L: Lex, T: Parse> Parse for Skip<L, T> {
    type Output = <T as Parse>::Output;

    fn parse<'i>(&mut self, input: &'i str) -> crate::ParseResult<'i, Self::Output> {
        let (output, remaining) = self.item.parse(input)?;
        let (_, remaining) = self.lexer.lex(remaining)?;

        Ok((output, remaining))
    }
}
