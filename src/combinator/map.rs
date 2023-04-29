use std::{any::type_name, fmt};

use crate::{Lex, Parse};

#[derive(Clone)]
pub struct Map<L, F> {
    lexer: L,
    f: F,
}

pub fn map<L, F, O>(lexer: L, f: F) -> Map<L, F>
where
    F: Fn(&str) -> O,
{
    Map { lexer, f }
}

impl<L: Lex, F, O> Parse for Map<L, F>
where
    F: Fn(&str) -> O,
{
    type Output = O;

    fn parse<'i>(&self, input: &'i str) -> crate::ParseResult<'i, Self::Output> {
        let (matched, remaining) = self.lexer.lex(input).map_err(|_| crate::Error::NoMatch)?;
        let output = (self.f)(matched);

        Ok((output, remaining))
    }
}

#[derive(Clone)]
pub struct TryMap<L, F> {
    lexer: L,
    f: F,
}

pub fn try_map<L, F, O, E>(lexer: L, f: F) -> TryMap<L, F>
where
    F: Fn(&str) -> Result<O, E>,
{
    TryMap { lexer, f }
}

impl<L: Lex, F, O, E> Parse for TryMap<L, F>
where
    F: Fn(&str) -> Result<O, E>,
{
    type Output = O;

    fn parse<'i>(&self, input: &'i str) -> crate::ParseResult<'i, Self::Output> {
        let (matched, remaining) = self.lexer.lex(input).map_err(|_| crate::Error::NoMatch)?;
        let output = (self.f)(matched).map_err(|_| crate::Error::FailedConversion)?;

        Ok((output, remaining))
    }
}

impl<L, F, O> fmt::Debug for Map<L, F>
where
    L: fmt::Debug,
    F: Fn(&str) -> O,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Map<{:?} -> {}>", self.lexer, type_name::<O>())
    }
}

impl<L, F, O, E> fmt::Debug for TryMap<L, F>
where
    L: fmt::Debug,
    F: Fn(&str) -> Result<O, E>,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "TryMap<{:?} -> {}>",
            self.lexer,
            type_name::<Result<O, E>>()
        )
    }
}
