use std::fmt;
use std::ops::ControlFlow;

use crate::{end, Error, Lex, LexResult, Parse, ParseResult};

use super::{many, traits::*, Many};

/// This combinator is returned by [`all()`]. See it's documentation for more details.
#[derive(Clone)]
pub struct All<T, C> {
    many: Many<T, C>,
}

impl<T, C> Sequence for All<T, C> {
    fn while_condition(&self, input: &str, _count: usize) -> bool {
        end().lex(input).is_err()
    }

    fn error_condition(&self, input: &str, count: usize) -> bool {
        self.many.error_condition(input, count) || end().lex(input).is_err()
    }
}

impl<T, C1> Collect for All<T, C1> {
    type Output<C> = All<T, C>;

    fn collect<C2>(self) -> Self::Output<C2>
    where
        Self: Sized,
    {
        let All { many } = self;

        let new = many.collect::<C2>();

        All { many: new }
    }
}

impl<P, C> ParseSequence<C> for All<P, C>
where
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
        error: &mut Option<Error<'i>>,
        outputs: &mut C,
    ) -> ControlFlow<(), &'i str> {
        self.many
            .parse_one(input, working_input, count, offset, error, outputs)
    }
}

impl<P, C> Parse for All<P, C>
where
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
                .unwrap_or_else(|| crate::Error::no_match(working_input))
                .offset(input))
        } else {
            Ok((outputs, &input[offset..]))
        }
    }
}

impl<L, C> LexSequence for All<L, C>
where
    L: Lex,
{
    type Lexer = L;

    fn lex_one<'i>(
        &self,
        input: &'i str,
        working_input: &mut &'i str,
        count: &mut usize,
        offset: &mut usize,
        error: &mut Option<Error<'i>>,
    ) -> ControlFlow<(), &'i str> {
        self.many
            .lex_one(input, working_input, count, offset, error)
    }
}

impl<L: Lex, C> Lex for All<L, C> {
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
                .unwrap_or_else(|| crate::Error::no_match(working_input))
                .offset(input))
        } else {
            Ok(input.split_at(offset))
        }
    }
}

/// Creates a combinator that applies a given parser or lexer multiple times until End of Input is seen, or else fails because the end of input was not seen.
pub fn all<T, O>(min: usize, item: T) -> All<T, Vec<O>> {
    All {
        many: many(min.., item),
    }
}

impl<T, C> fmt::Debug for All<T, C>
where
    T: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "All({:?})", self.many)
    }
}

#[cfg(test)]
mod tests {
    use crate::{char, int, sequence_traits::*, Parse};

    #[test]
    fn test_all() -> Result<(), crate::ErrorOwned> {
        let csv_parser = int::<u8>().all(1).delimiter(char(','));

        let (output, remaining) = csv_parser.parse("1,2,3")?;
        assert_eq!(output, vec![1, 2, 3]);
        assert_eq!(remaining, "");

        let result = csv_parser.parse("1,2,3foo");
        assert_eq!(result.unwrap_err().remaining, "foo");

        Ok(())
    }
}
