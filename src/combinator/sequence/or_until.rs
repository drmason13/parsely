use std::ops::ControlFlow;
use std::{fmt, ops::RangeBounds};

use crate::{InProgressError, Lex, LexResult, Parse, ParseResult};

use super::{min_max_from_bounds, traits::*, Delimited, Many};

/// This combinator is returned by [`or_until()`]. See it's documentation for more details.
#[derive(Clone)]
pub struct OrUntil<L, T, C> {
    until: L,
    many: Many<T, C>,
}

impl<L: Lex, T, C> OrUntil<L, T, C> {
    /// Creates a new OrUntil combinator, this is a low level method.
    /// Prefer using [`many(min..=max).or_until()`](crate::combinator::Many::or_until) instead
    pub fn new(until: L, item: T, min: usize, max: usize) -> Self {
        OrUntil {
            until,
            many: Many::new(item, min, max),
        }
    }

    /// Creates a new parser that matches the same sequence, but expects the input to be separated by `delimiter`.
    ///
    /// A trailing match is optional, so this is suitable for parsing separated lists.
    ///
    /// # Examples
    ///
    /// Basic usage:
    ///
    /// ```
    /// use parsely::{char, int, Parse};
    ///
    /// let csv_parser = int::<u8>().all(1).delimiter(char(','));
    ///
    /// let (output, remaining) = csv_parser.parse("1,2,3").expect("ok okay geez");
    /// assert_eq!(output, vec![1, 2, 3]);
    /// assert_eq!(remaining, "");
    ///
    /// let result = csv_parser.parse("1,2,3foo");
    /// assert_eq!(result.unwrap_err().remaining, "foo");
    /// # Ok::<(), parsely::InProgressError>(())
    /// ```
    pub fn delimiter<D: Lex>(self, delimiter: D) -> Delimited<D, Self, C>
    where
        Self: Sized,
    {
        Delimited::new(self, delimiter)
    }

    /// This method works the same way as [`Many::collect`](crate::combinator::Many::collect()). See it’s documentation for more details.
    #[inline(always)]
    pub fn collect<C2>(self) -> OrUntil<L, T, C2>
    where
        Self: Sized,
    {
        <Self as Collect>::collect::<C2>(self)
    }
}

impl<L, T, C> Sequence for OrUntil<L, T, C>
where
    L: Lex,
{
    fn while_condition(&self, input: &str, _count: usize) -> bool {
        self.until.lex(input).is_err()
    }

    fn error_condition(&self, input: &str, count: usize) -> bool {
        self.many.error_condition(input, count)
    }
}

impl<L, T, C1> Collect for OrUntil<L, T, C1> {
    type Output<C> = OrUntil<L, T, C>;

    fn collect<C2>(self) -> Self::Output<C2>
    where
        Self: Sized,
    {
        let OrUntil { until, many } = self;

        let new = many.collect::<C2>();

        OrUntil { until, many: new }
    }
}

impl<L, P, C> ParseSequence<C> for OrUntil<L, P, C>
where
    L: Lex,
    P: Parse,
    C: Default + Extend<<P as Parse>::Output>,
{
    type Parser = P;

    fn parse_one<'i>(
        &self,
        input: &'i str,
        working_input: &mut &'i str,
        count: &mut usize,
        offset: &mut usize,
        error: &mut Option<InProgressError<'i>>,
        outputs: &mut C,
    ) -> ControlFlow<(), &'i str> {
        self.many
            .parse_one(input, working_input, count, offset, error, outputs)
    }
}

impl<L, P, C> Parse for OrUntil<L, P, C>
where
    L: Lex,
    P: Parse,
    C: Default + Extend<<P as Parse>::Output>,
{
    type Output = C;

    fn parse<'i>(&self, input: &'i str) -> ParseResult<'i, Self::Output> {
        let mut working_input = input;
        let mut count = 0;
        let mut offset = 0;
        let mut error = None;
        let mut outputs = C::default();

        while self.while_condition(working_input, count) {
            match self.parse_one(
                input,
                &mut working_input,
                &mut count,
                &mut offset,
                &mut error,
                &mut outputs,
            ) {
                ControlFlow::Continue(_) => continue,
                ControlFlow::Break(_) => break,
            }
        }

        if self.error_condition(working_input, count) {
            Err(error
                .unwrap_or_else(|| crate::InProgressError::no_match(working_input))
                .offset(input))
        } else {
            Ok((outputs, &input[offset..]))
        }
    }
}

impl<U, L, C> LexSequence for OrUntil<U, L, C>
where
    U: Lex,
    L: Lex,
{
    type Lexer = L;

    fn lex_one<'i>(
        &self,
        input: &'i str,
        working_input: &mut &'i str,
        count: &mut usize,
        offset: &mut usize,
        error: &mut Option<InProgressError<'i>>,
    ) -> ControlFlow<(), &'i str> {
        self.many
            .lex_one(input, working_input, count, offset, error)
    }
}

impl<U: Lex, L: Lex, C> Lex for OrUntil<U, L, C> {
    fn lex<'i>(&self, input: &'i str) -> LexResult<'i> {
        let mut working_input = input;
        let mut count = 0;
        let mut offset = 0;
        let mut error = None;

        while self.while_condition(working_input, count) {
            match self.lex_one(
                input,
                &mut working_input,
                &mut count,
                &mut offset,
                &mut error,
            ) {
                ControlFlow::Continue(_) => continue,
                ControlFlow::Break(_) => break,
            }
        }

        if self.error_condition(working_input, count) {
            Err(error
                .unwrap_or_else(|| crate::InProgressError::no_match(working_input))
                .offset(input))
        } else {
            Ok(input.split_at(offset))
        }
    }
}

/// Creates a combinator that applies a given parser or lexer multiple times until a given lexer matches the remaining input.
pub fn or_until<L, T, C>(range: impl RangeBounds<usize>, until: L, item: T) -> OrUntil<L, T, C> {
    let (min, max) = min_max_from_bounds(range);
    OrUntil {
        until,
        many: Many::new(item, min, max),
    }
}

impl<L, T, C> OrUntil<L, T, C> {}

impl<L, T, C> fmt::Debug for OrUntil<L, T, C>
where
    T: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "OrUntil({:?})", self.many)
    }
}

#[cfg(test)]
mod tests {
    use crate::{char, end, int, Parse};

    #[test]
    fn test_or_until() -> Result<(), crate::Error> {
        let csv_parser = int::<u8>().many(2..=3).or_until(end()).delimiter(char(','));

        let (output, remaining) = csv_parser.parse("1,2")?;
        assert_eq!(output, vec![1, 2]);
        assert_eq!(remaining, "");

        let result = csv_parser.parse("1,foo");
        assert_eq!(result.unwrap_err().remaining, "foo");

        Ok(())
    }
}
