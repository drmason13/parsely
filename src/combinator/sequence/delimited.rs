//! [`.many(..).delimited(delimiter)`](super::many::Many::delimiter()) will expect a delimiter in between each item.

use std::{marker::PhantomData, ops::RangeBounds};

use crate::{Lex, LexResult, Parse, ParseResult};

use super::min_max_from_bounds;

/// This combinator is returned by [`Many::delimiter()`](super::many::Many::delimiter()). See it's documentation for more details.
#[derive(Debug, Clone)]
pub struct Delimited<L, T, C> {
    delimiter: L,
    item: T,
    min: usize,
    max: usize,
    collection: PhantomData<C>,
}

impl<L: Lex, T, C> Delimited<L, T, C> {
    /// Returns a new Delimited combinator. See also [`delimited()`]
    pub fn new(min: usize, max: usize, item: T, delimiter: L) -> Self {
        Delimited {
            min,
            max,
            item,
            delimiter,
            collection: PhantomData::<C>,
        }
    }
}

impl<L, T, C> Parse for Delimited<L, T, C>
where
    T: Parse,
    L: Lex,
    C: Default + Extend<<T as Parse>::Output>,
{
    type Output = C;

    fn parse<'i>(&self, input: &'i str) -> ParseResult<'i, Self::Output> {
        let mut count = 0;
        let mut offset = 0;
        let mut working_input = input;

        let mut outputs = C::default();

        while count < self.max {
            match self.item.parse(working_input) {
                Ok((output, remaining)) => match self.delimiter.lex(remaining) {
                    Ok((_, remaining)) => {
                        count += 1;
                        offset = input.len() - remaining.len();
                        outputs.extend(Some(output));
                        working_input = remaining;
                    }
                    Err(_) => {
                        count += 1;
                        outputs.extend(Some(output));
                        offset = input.len() - remaining.len();

                        break;
                    }
                },
                Err(_) => break,
            }
        }

        if count < self.min {
            Err(crate::Error::no_match(input))
        } else {
            Ok((outputs, &input[offset..]))
        }
    }
}

impl<L, T, C> Lex for Delimited<L, T, C>
where
    T: Lex,
    L: Lex,
{
    fn lex<'i>(&self, input: &'i str) -> LexResult<'i> {
        let mut count = 0;
        let mut offset = 0;
        let mut working_input = input;

        while count < self.max {
            match self.item.lex(working_input) {
                Ok((_, remaining)) => match self.delimiter.lex(remaining) {
                    Ok((_, remaining)) => {
                        count += 1;
                        offset = input.len() - remaining.len();
                        working_input = remaining;
                    }
                    Err(_) => {
                        count += 1;
                        offset = input.len() - remaining.len();

                        break;
                    }
                },
                Err(_) => break,
            }
        }

        if count < self.min {
            Err(crate::Error::no_match(input))
        } else {
            Ok((&input[..offset], &input[offset..]))
        }
    }
}

impl<L, T, O> Delimited<L, T, Vec<O>> {
    /// This method works the same way as [`Many::collect`](crate::combinator::Many::collect()). See it’s documentation for more details.
    pub fn collect<C>(self) -> Delimited<L, T, C>
    where
        Self: Sized,
        C: Extend<O>,
    {
        let Delimited {
            delimiter,
            item,
            min,
            max,
            collection: _,
        } = self;

        Delimited {
            delimiter,
            item,
            min,
            max,
            collection: PhantomData::<C>,
        }
    }
}

/// Creates a parser/lexer that expects a delimiter in between each item.
///
/// Like [`many()`](crate::combinator::many()) this function takes a range to specify a minimum and maximum number of matches.
/// See the module docs of [`many`](crate::combinator::many) for more details.
pub fn delimited<L: Lex, T, C>(
    delimiter: L,
    range: impl RangeBounds<usize>,
    item: T,
) -> Delimited<L, T, C> {
    let (min, max) = min_max_from_bounds(range);

    Delimited::new(min, max, item, delimiter)
}
