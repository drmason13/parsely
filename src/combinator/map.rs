use std::{any::type_name, fmt};

use crate::{Lex, Parse, ParseError};

#[derive(Clone)]
pub struct Map<L, F> {
    lexer: L,
    f: F,
}

pub fn map<L, F, O, E>(lexer: L, f: F) -> Map<L, F>
where
    F: Fn(&str) -> Result<O, E>,
{
    Map { lexer, f }
}

impl<L: Lex, F, O, E> Parse for Map<L, F>
where
    F: Fn(&str) -> Result<O, E>,
    E: std::error::Error,
{
    type Output = O;

    fn parse<'i>(&mut self, input: &'i str) -> crate::ParseResult<'i, Self::Output> {
        let (matched, remaining) = self.lexer.lex(input).map_err(|_| ParseError::NoMatch)?;
        let output = (self.f)(matched).map_err(|_| ParseError::FailedConversion)?;

        Ok((output, remaining))
    }
}

impl<L, F, O, E> fmt::Debug for Map<L, F>
where
    L: fmt::Debug,
    F: Fn(&str) -> Result<O, E>,
    E: std::error::Error,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Map<{:?} -> {}>", self.lexer, type_name::<O>())
    }
}
