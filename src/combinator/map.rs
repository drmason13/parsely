use std::{any::type_name, fmt};

use crate::{error::result_ext::*, Error, Lex, LexResult, Parse};

/// This combinator is returned by [`map()`]. See its documentation for more details.
#[derive(Clone)]
pub struct Map<L, F> {
    lexer: L,
    f: F,
}

/// This combinator is used to build a custom parser from a lexer by mapping the matched &str to an output type.
///
/// See [`try_map`] if the conversion may fail.
///
/// See [`Lex::map()`] for more details and examples.
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
        let (matched, remaining) = self.lexer.lex(input).offset(input)?;
        let output = (self.f)(matched);

        Ok((output, remaining))
    }
}

/// This combinator is returned by [`try_map()`]. See its documentation for more details.
#[derive(Clone)]
pub struct TryMap<L, F> {
    lexer: L,
    f: F,
}

/// Like [`map`] except the mapping function is fallible, returning Result.
///
/// See [`Lex::try_map()`] for more details and examples.
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
        let (matched, remaining) = self.lexer.lex(input).offset(input)?;
        let output = (self.f)(matched).fail_conversion(input)?;

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

/// This combinator is returned by [`map_err()`]. See its documentation for more details.
pub struct MapError<T, E: std::error::Error> {
    inner: T,
    f: Box<dyn Fn(&Error) -> Option<E>>,
}

/// Map [`parsely::Error`](crate::Error)s into a [`Custom`](crate::ErrorReason::Custom) error variant.
/// Used to inject user defined errors during parsing.
///
/// See [`Lex::map_err()`] for more details and examples.
pub fn map_err<T, E, F>(inner: T, f: F) -> MapError<T, E>
where
    E: std::error::Error,
    F: Fn(&Error) -> Option<E> + 'static,
{
    MapError {
        inner,
        f: Box::new(f),
    }
}

impl<T, E> Lex for MapError<T, E>
where
    T: Lex,
    E: std::error::Error + 'static,
{
    fn lex<'i>(&self, input: &'i str) -> LexResult<'i> {
        self.inner.lex(input).map_err(|e| {
            if let Some(error) = (self.f)(&e) {
                e.map_err(error)
            } else {
                e
            }
        })
    }
}

impl<T, O, E> Parse for MapError<T, E>
where
    T: Parse<Output = O>,
    E: std::error::Error + 'static,
{
    type Output = O;

    fn parse<'i>(&self, input: &'i str) -> crate::ParseResult<'i, Self::Output> {
        self.inner.parse(input).map_err(|e| {
            if let Some(error) = (self.f)(&e) {
                e.map_err(error)
            } else {
                e
            }
        })
    }
}
