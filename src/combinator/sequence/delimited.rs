//! [`.many(..).delimited(delimiter)`](super::many::Many::delimiter()) will expect a delimiter in between each item.

use std::ops::RangeBounds;

use crate::{Lex, LexResult, Parse, ParseResult};

use super::min_max_from_bounds;

/// This combinator is returned by [`Many::delimiter()`](super::many::Many::delimiter()). See it's documentation for more details.
#[derive(Debug, Clone)]
pub struct Delimited<L, T> {
    delimiter: L,
    item: T,
    min: usize,
    max: usize,
}

impl<L: Lex, T> Delimited<L, T> {
    /// Returns a new Delimited combinator. See also [`delimited()`]
    pub fn new(min: usize, max: usize, item: T, delimiter: L) -> Self {
        Delimited {
            min,
            max,
            item,
            delimiter,
        }
    }
}

impl<L, T> Parse for Delimited<L, T>
where
    T: Parse,
    L: Lex,
{
    type Output = Vec<<T as Parse>::Output>;

    fn parse<'i>(&self, input: &'i str) -> ParseResult<'i, Self::Output> {
        let mut count = 0;
        let mut offset = 0;
        let mut working_input = input;

        let capacity = std::cmp::max(self.min, 4);

        let mut outputs = Vec::with_capacity(capacity);

        while count < self.max {
            match self.item.parse(working_input) {
                Ok((output, remaining)) => match self.delimiter.lex(remaining) {
                    Ok((_, remaining)) => {
                        count += 1;
                        offset = input.len() - remaining.len();
                        outputs.push(output);
                        working_input = remaining;
                    }
                    Err(_) => {
                        count += 1;
                        outputs.push(output);
                        offset = input.len() - remaining.len();

                        break;
                    }
                },
                Err(_) => break,
            }
        }

        if count < self.min {
            Err(crate::Error::NoMatch)
        } else {
            Ok((outputs, &input[offset..]))
        }
    }
}

impl<L, T> Lex for Delimited<L, T>
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
            Err(crate::Error::NoMatch)
        } else {
            Ok((&input[..offset], &input[offset..]))
        }
    }
}

/// Creates a parser/lexer that expects a delimiter in between each item.
///
/// Like [`many()`](crate::combinator::many()) this function takes a range to specify a minimum and maximum number of matches.
/// See the module docs of [`many`](crate::combinator::many) for more details.
pub fn delimited<L: Lex, T>(
    delimiter: L,
    range: impl RangeBounds<usize>,
    item: T,
) -> Delimited<L, T> {
    let (min, max) = min_max_from_bounds(range);

    Delimited::new(min, max, item, delimiter)
}
