use crate::{Lex, Parse};

#[derive(Debug, Clone)]
pub struct ThenSkip<L, T> {
    lexer: L,
    item: T,
}

/// *After* running the item (parser or lexer), this combinator will run the given lexer and discard its output.
///
/// If the lexer fails, it is still a parse failure. Use `.optional()` if the input to be skipped isn't required.
pub fn then_skip<L: Lex, T>(lexer: L, item: T) -> ThenSkip<L, T> {
    ThenSkip { lexer, item }
}

impl<L: Lex, T: Lex> Lex for ThenSkip<L, T> {
    fn lex<'i>(&mut self, input: &'i str) -> crate::LexResult<'i> {
        let (output, remaining) = self.item.lex(input)?;
        let (_, remaining) = self.lexer.lex(remaining)?;
        Ok((output, remaining))
    }
}

impl<L: Lex, T: Parse> Parse for ThenSkip<L, T> {
    type Output = <T as Parse>::Output;

    fn parse<'i>(&mut self, input: &'i str) -> crate::ParseResult<'i, Self::Output> {
        let (output, remaining) = self.item.parse(input)?;
        let (_, remaining) = self.lexer.lex(remaining)?;

        Ok((output, remaining))
    }
}

#[derive(Debug, Clone)]
pub struct SkipThen<L, T> {
    lexer: L,
    item: T,
}

/// *Before* running the item (parser or lexer), this combinator will run the given lexer and discard its output.
///
/// If the lexer fails, it is still a parse failure. Use `.optional()` if the input to be skipped isn't required.
pub fn skip_then<L: Lex, T>(lexer: L, item: T) -> SkipThen<L, T> {
    SkipThen { lexer, item }
}

impl<L: Lex, T: Lex> Lex for SkipThen<L, T> {
    fn lex<'i>(&mut self, input: &'i str) -> crate::LexResult<'i> {
        let (_, remaining) = self.lexer.lex(input)?;
        let (output, remaining) = self.item.lex(remaining)?;
        Ok((output, remaining))
    }
}

impl<L: Lex, T: Parse> Parse for SkipThen<L, T> {
    type Output = <T as Parse>::Output;

    fn parse<'i>(&mut self, input: &'i str) -> crate::ParseResult<'i, Self::Output> {
        let (_, remaining) = self.lexer.lex(input)?;
        let (output, remaining) = self.item.parse(remaining)?;

        Ok((output, remaining))
    }
}
