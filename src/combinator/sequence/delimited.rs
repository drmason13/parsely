//! [`.many(..).delimited(delimiter)`](super::many::Many::delimiter()) will expect a delimiter in between each item.

use std::{
    marker::PhantomData,
    ops::{ControlFlow, RangeBounds},
};

use crate::{Lex, LexResult, Parse, ParseResult};

use super::{many, traits::*, Many};

/// This combinator is returned by [`Many::delimiter()`](super::many::Many::delimiter()). See it's documentation for more details.
#[derive(Debug, Clone)]
pub struct Delimited<L, S, C> {
    delimiter: L,
    sequencer: S,
    collection: PhantomData<C>,
}

impl<L: Lex, S, C> Delimited<L, S, C>
where
    S: Sequence,
{
    /// Returns a new Delimited combinator. See also [`delimited()`]
    pub fn new(sequencer: S, delimiter: L) -> Self {
        Delimited {
            sequencer,
            delimiter,
            collection: PhantomData::<C>,
        }
    }

    /// This method works the same way as [`Many::collect`](crate::combinator::Many::collect()). See it’s documentation for more details.
    pub fn collect<C2>(self) -> Delimited<L, <S as Collect>::Output<C2>, C2>
    where
        Self: Sized,
    {
        let sequencer = self.sequencer.collect::<C2>();

        let Delimited {
            delimiter,
            sequencer: _,
            collection: _,
        } = self;

        Delimited {
            delimiter,
            sequencer,
            collection: PhantomData::<C2>,
        }
    }
}

impl<L, S, C> Parse for Delimited<L, S, C>
where
    S: ParseSequence<C>,
    L: Lex,
    C: Default + Extend<<<S as ParseSequence<C>>::Parser as Parse>::Output>,
{
    type Output = C;

    fn parse<'i>(&self, input: &'i str) -> ParseResult<'i, Self::Output> {
        let mut working_input = input;
        let mut count = 0;
        let mut offset = 0;
        let mut error = None;
        let mut outputs = C::default();

        while self.sequencer.while_condition(working_input, count) {
            match self.sequencer.parse_one(
                input,
                &mut working_input,
                &mut count,
                &mut offset,
                &mut error,
                &mut outputs,
            ) {
                ControlFlow::Continue(remaining) => match self.delimiter.lex(remaining) {
                    Ok((_, remaining)) => {
                        // only need to skip over the delimiter, everything else is done by the sequencer
                        offset = input.len() - remaining.len();
                        working_input = remaining;
                    }
                    Err(_) => break,
                },
                ControlFlow::Break(_) => break,
            }
        }

        if self.sequencer.error_condition(working_input, count) {
            Err(error
                .unwrap_or_else(|| crate::InProgressError::no_match(working_input))
                .offset(input))
        } else {
            Ok((outputs, &input[offset..]))
        }
    }
}

impl<L, S, C> Lex for Delimited<L, S, C>
where
    S: LexSequence,
    L: Lex,
{
    fn lex<'i>(&self, input: &'i str) -> LexResult<'i> {
        let mut working_input = input;
        let mut count = 0;
        let mut offset = 0;
        let mut error = None;

        while self.sequencer.while_condition(working_input, count) {
            match self.sequencer.lex_one(
                input,
                &mut working_input,
                &mut count,
                &mut offset,
                &mut error,
            ) {
                ControlFlow::Continue(remaining) => match self.delimiter.lex(remaining) {
                    Ok((_, remaining)) => {
                        // only need to skip over the delimiter, everything else is done by the sequencer
                        offset = input.len() - remaining.len();
                        working_input = remaining;
                    }
                    Err(_) => break,
                },
                ControlFlow::Break(_) => break,
            }
        }

        if self.sequencer.error_condition(working_input, count) {
            Err(error
                .unwrap_or_else(|| crate::InProgressError::no_match(working_input))
                .offset(input))
        } else {
            Ok((&input[..offset], &input[offset..]))
        }
    }
}

/// Creates a parser/lexer that expects a delimiter in between each item.
///
/// Like [`many()`](crate::combinator::many()) this function takes a range to specify a minimum and maximum number of matches.
/// See the docs of [`many`](crate::combinator::many()) for more details.
pub fn delimited<L: Lex, T>(
    delimiter: L,
    range: impl RangeBounds<usize>,
    item: T,
) -> Delimited<L, Many<T, Vec<T>>, Vec<T>> {
    let sequencer = many(range, item);

    Delimited::new(sequencer, delimiter)
}
