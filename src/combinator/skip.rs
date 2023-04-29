use crate::{Lex, Parse};

/// This combinator is returned by [`then_skip()`]. See it's documentation for more details.
#[derive(Debug, Clone)]
pub struct ThenSkip<L, T> {
    lexer: L,
    item: T,
}

/// *After* running the item (parser or lexer), this combinator will run the given lexer and discard its output.
///
/// If the lexer fails, it is still a parse failure. Use `.optional()` if the input to be skipped isn't required.
///
/// This combinator can be chained using [`Parse::then_skip()`].
//
// TODO: what actually happens if you do a then_skip(lexer_a, lexer_b) ???
pub fn then_skip<L: Lex, T>(lexer: L, item: T) -> ThenSkip<L, T> {
    ThenSkip { lexer, item }
}

impl<L: Lex, T: Lex> Lex for ThenSkip<L, T> {
    fn lex<'i>(&self, input: &'i str) -> crate::LexResult<'i> {
        let (output, remaining) = self.item.lex(input)?;
        let (_, remaining) = self.lexer.lex(remaining)?;
        Ok((output, remaining))
    }
}

impl<L: Lex, T: Parse> Parse for ThenSkip<L, T> {
    type Output = <T as Parse>::Output;

    fn parse<'i>(&self, input: &'i str) -> crate::ParseResult<'i, Self::Output> {
        let (output, remaining) = self.item.parse(input)?;
        let (_, remaining) = self.lexer.lex(remaining)?;

        Ok((output, remaining))
    }
}

/// This combinator is returned by [`skip_then()`]. See it's documentation for more details.
#[derive(Debug, Clone)]
pub struct SkipThen<L, T> {
    lexer: L,
    item: T,
}

/// *Before* running the item (parser or lexer), this combinator will run the given lexer and discard its output.
///
/// If the lexer fails, it is still a parse failure. Use `.optional()` if the input to be skipped isn't required.
///
/// This combinator can be chained using [`Lex::skip_then()`].
pub fn skip_then<L: Lex, T>(lexer: L, item: T) -> SkipThen<L, T> {
    SkipThen { lexer, item }
}

impl<L: Lex, T: Lex> Lex for SkipThen<L, T> {
    fn lex<'i>(&self, input: &'i str) -> crate::LexResult<'i> {
        let (_, remaining) = self.lexer.lex(input)?;
        let (output, remaining) = self.item.lex(remaining)?;
        Ok((output, remaining))
    }
}

impl<L: Lex, T: Parse> Parse for SkipThen<L, T> {
    type Output = <T as Parse>::Output;

    fn parse<'i>(&self, input: &'i str) -> crate::ParseResult<'i, Self::Output> {
        let (_, remaining) = self.lexer.lex(input)?;
        let (output, remaining) = self.item.parse(remaining)?;

        Ok((output, remaining))
    }
}
