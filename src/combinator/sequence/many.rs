use std::marker::PhantomData;
use std::ops::ControlFlow;
use std::{fmt, ops::RangeBounds};

use crate::{result_ext::*, Error, Lex, LexResult, Parse, ParseResult};

use super::{min_max_from_bounds, or_until, traits::*, Delimited, OrUntil, MAX_LIMIT};

/// This type alias is used where [`Many`] requires a generic type to collect into that we can ignore because we're lexing.
pub(crate) type LexMany<T> = Many<T, Vec<()>>;

/// This combinator is returned by [`many()`]. See it's documentation for more details.
#[derive(Clone)]
pub struct Many<T, C> {
    /// The lexer/parser to be repeated.
    item: T,

    /// The minimum number of times the parser must match for the parse to succeed.
    ///
    /// If the parser matches fewer than min times, the overall parse fails, and no input is consumed.
    min: usize,

    /// The maximum number of times the parser will attempt to match.
    ///
    /// The parser will never match more than max times, because it doesn't try to.
    ///
    /// To enforce that input is fully consumed, see [`crate::lexer::end()`]
    max: usize,

    collection: PhantomData<C>,
}

impl<T, C> Many<T, C> {
    /// Creates a new Many combinator, this is a low level method.
    /// Prefer using [`many(min..=max, item)`](many) instead
    pub fn new(item: T, min: usize, max: usize) -> Self {
        Many {
            item,
            min,
            max,
            collection: PhantomData::<C>,
        }
    }

    /// Returns a new sequence combinator that returns early if the given lexer matches the remaining input
    pub fn or_until<L: Lex>(self, until: L) -> OrUntil<L, T, C> {
        or_until(self.min..self.max, until, self.item)
    }

    /// Creates a new parser that matches the same sequence, but expects the input to be separated by `delimiter`.
    ///
    /// A trailing match is optional, so this is suitable for parsing separated lists.
    ///
    /// # Examples
    ///
    /// ```
    /// use parsely::{int, Parse};
    ///
    /// let csv_parser = int::<u8>().all(1).delimiter(',');
    ///
    /// let (output, remaining) = csv_parser.parse("1,2,3").expect("ok okay geez");
    /// assert_eq!(output, vec![1, 2, 3]);
    /// assert_eq!(remaining, "");
    ///
    /// let result = csv_parser.parse("1,2,3foo");
    /// assert_eq!(result.unwrap_err().remaining, "foo");
    /// # Ok::<(), parsely::Error>(())
    /// ```
    pub fn delimiter<L: Lex>(self, delimiter: L) -> Delimited<L, Self, C>
    where
        Self: Sized,
    {
        Delimited::new(self, delimiter)
    }

    /// By default Many collects output into a [`Vec<T>`]. Use this method to tell [`Many`] to instead collect into a different type when parsing.
    ///
    /// The new collection type must implement [`Extend`]. This trait is implemented for most [`std::collections`] types.
    ///
    /// Specify the collection type to use with a turbofish. Rust is often not able to infer the type you want to collect into.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::collections::LinkedList;
    /// use parsely::{digit, Lex, Parse};
    ///
    /// let integers = digit().try_map(str::parse::<u8>).many(1..).collect::<LinkedList<u8>>();
    /// #
    /// # let (output, remaining) = integers.parse("123")?;
    /// # assert_eq!(output, {
    /// #    let mut linked_list = LinkedList::new();
    /// #    linked_list.push_back(1);
    /// #    linked_list.push_back(2);
    /// #    linked_list.push_back(3);
    /// #    linked_list
    /// # });
    /// # Ok::<(), parsely::Error>(())
    /// ```
    ///
    /// Collect into a HashMap:
    ///
    /// ```
    /// use std::collections::HashMap;
    /// use parsely::{any, int, Lex, Parse};
    ///
    /// let integers = any()
    ///     .map(str::to_string)
    ///     .then_skip(':')
    ///     .then(int::<u8>())
    ///     .many(1..)
    ///     .delimiter(',')
    ///     .collect::<HashMap<String, u8>>();
    ///
    /// let (output, remaining) = integers.parse("a:1,b:2,c:3")?;
    ///
    /// assert_eq!(output.get("a"), Some(&1));
    /// assert_eq!(output.get("b"), Some(&2));
    /// assert_eq!(output.get("c"), Some(&3));
    /// # assert_eq!(output, {
    /// #     let mut map = HashMap::new();
    /// #     map.insert("a".to_string(), 1);
    /// #     map.insert("b".to_string(), 2);
    /// #     map.insert("c".to_string(), 3);
    /// #     map
    /// # });
    /// # Ok::<(), parsely::Error>(())
    #[inline(always)]
    pub fn collect<C2>(self) -> Many<T, C2>
    where
        Self: Sized,
    {
        <Self as Collect>::collect::<C2>(self)
    }
}

impl<T, C> Sequence for Many<T, C> {
    fn while_condition(&self, _input: &str, count: usize) -> bool {
        count < self.max
    }

    fn error_condition(&self, _input: &str, count: usize) -> bool {
        count < self.min
    }
}

impl<T, C1> Collect for Many<T, C1> {
    type Output<C> = Many<T, C>;

    fn collect<C2>(self) -> Self::Output<C2>
    where
        Self: Sized,
    {
        let Many {
            item,
            min,
            max,
            collection: _,
        } = self;

        Many {
            item,
            min,
            max,
            collection: PhantomData::<C2>,
        }
    }
}

impl<P, C> ParseSequence<C> for Many<P, C>
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
        match self.item.parse(working_input).offset(input) {
            Ok((output, remaining)) => {
                *count += 1;
                *offset = input.len() - remaining.len();
                *working_input = remaining;
                outputs.extend(Some(output));
                ControlFlow::Continue(remaining)
            }
            Err(e) => {
                *error = Some(e);
                ControlFlow::Break(())
            }
        }
    }
}

impl<P, C> Parse for Many<P, C>
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

impl<L, C> LexSequence for Many<L, C>
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
        match self.item.lex(working_input).offset(input) {
            Ok((_, remaining)) => {
                *count += 1;
                *offset = input.len() - remaining.len();
                *working_input = remaining;
                ControlFlow::Continue(remaining)
            }
            Err(e) => {
                *error = Some(e);
                ControlFlow::Break(())
            }
        }
    }
}

impl<L: Lex, C> Lex for Many<L, C> {
    fn lex<'i>(&self, input: &'i str) -> LexResult<'i> {
        let mut working_input = input;
        let mut count = 0;
        let mut offset = 0;
        let mut error = None;

        while self.while_condition(input, count) {
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

/// Creates a combinator that applies a given parser or lexer multiple times.
///
/// This function takes a Range-like argument as a succint description of start and end bounds.
///
/// The start bound becomes the minimum number of times the parser must match to succeed.
///
/// The end bound becomes the maximum number of times the parser will attempt to parse.
///
/// This combinator can be chained using [`Parse::many()`] or [`Lex::many()`].
///
/// # Examples
///
/// ```
/// use parsely::{digit, Lex};
/// use parsely::combinator::many;
///
/// // these are equivalent
/// let zero_or_more_digits = many::<_, ()>(.., digit());
/// let zero_or_more_digits = many::<_, ()>(0.., digit());
///
/// let (output, remaining) = zero_or_more_digits.lex("123")?;
/// assert_eq!(output, "123");
/// assert_eq!(remaining, "");
///
/// let (output, remaining) = zero_or_more_digits.lex("abc")?;
/// assert_eq!(output, "");
/// assert_eq!(remaining, "abc");
///
/// let one_or_more_digits = many::<_, ()>(1.., digit());
///
/// let result = one_or_more_digits.lex("abc");
/// assert!(result.is_err());
/// # Ok::<(), parsely::Error>(())
/// ```
///
/// Chain with [`Lex::many()`]:
///
/// ```
/// use parsely::{digit, Lex};
///
/// let zero_or_more_digits = digit().many(0..);
///
/// # let (output, remaining) = zero_or_more_digits.lex("123")?;
/// # assert_eq!(output, "123");
/// # assert_eq!(remaining, "");
/// #
/// # let (output, remaining) = zero_or_more_digits.lex("abc")?;
/// # assert_eq!(output, "");
/// # assert_eq!(remaining, "abc");
/// # Ok::<(), parsely::Error>(())
/// ```
///
/// Min and Max:
///
/// ```
/// use parsely::{digit, Lex};
///
/// let three_or_four_digits = digit().many(3..=4);
///
/// let (output, remaining) = three_or_four_digits.lex("123")?;
/// assert_eq!(output, "123");
/// assert_eq!(remaining, "");
///
/// let result = three_or_four_digits.lex("12");
/// assert!(result.is_err());
///
/// let (output, remaining) = three_or_four_digits.lex("12345")?;
/// assert_eq!(output, "1234");
/// assert_eq!(remaining, "5");
/// # Ok::<(), parsely::Error>(())
/// ```
pub fn many<T, O>(range: impl RangeBounds<usize>, item: T) -> Many<T, Vec<O>> {
    let (min, max) = min_max_from_bounds(range);
    Many {
        item,
        min,
        max,
        collection: PhantomData::<Vec<O>>,
    }
}

/// Creates a combinator that applies a given parser or lexer multiple times.
///
/// This function takes a Range-like argument as a succint description of start and end bounds.
///
/// The start bound becomes the minimum number of times the parser must match to succeed.
///
/// The end bound becomes the maximum number of times the parser will attempt to parse.
pub fn count<T, O>(count: usize, item: T) -> Many<T, Vec<O>> {
    Many {
        item,
        min: count,
        max: count,
        collection: PhantomData::<Vec<O>>,
    }
}

impl<T, C> fmt::Debug for Many<T, C>
where
    T: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.max == MAX_LIMIT {
            write!(f, "Many({}.., {:?})", self.min, self.item)
        } else {
            write!(f, "Many({}..={}, {:?})", self.min, self.max, self.item)
        }
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use super::*;
    use crate::test_utils::*;

    #[derive(PartialEq, Debug, Clone)]
    struct A;
    impl FromStr for A {
        type Err = crate::ErrorOwned;

        fn from_str(s: &str) -> Result<Self, Self::Err> {
            if s == "a" {
                Ok(A)
            } else {
                Err(crate::Error::no_match(s))?
            }
        }
    }

    #[test]
    fn min_and_max_parse() {
        let a_parser = || 'a'.try_map(A::from_str);

        test_parser_batch(
            "1..=3 matches 1, 2 or 3 times",
            many(1..=3, a_parser()),
            &[
                ("", None, ""), //
                ("abcd", Some(vec![A]), "bcd"),
                ("zzz", None, "zzz"),
                ("zaa", None, "zaa"),
                ("aaaaa", Some(vec![A, A, A]), "aa"),
                ("aa|aaa", Some(vec![A, A]), "|aaa"),
            ],
        );

        test_parser_batch(
            ".. matches any number of times",
            many(.., a_parser()),
            &[
                ("", Some(vec![]), ""), //
                ("abcd", Some(vec![A]), "bcd"),
                ("zzz", Some(vec![]), "zzz"),
                ("zaa", Some(vec![]), "zaa"),
                ("aaaaa", Some(vec![A, A, A, A, A]), ""),
                ("aa|aaa", Some(vec![A, A]), "|aaa"),
            ],
        );

        test_parser_batch(
            "3..5 matches 3 or 4 times",
            many(3..5, a_parser()),
            &[
                ("", None, ""), //
                ("abcd", None, "abcd"),
                ("zzz", None, "zzz"),
                ("zaa", None, "zaa"),
                ("aaaaa", Some(vec![A, A, A, A]), "a"),
                ("aa|aaa", None, "aa|aaa"),
                ("a|aaaa", None, "a|aaaa"),
                ("aaa|aa", Some(vec![A, A, A]), "|aa"),
            ],
        );
    }

    #[test]
    fn min_and_max_lex() {
        test_lexer_batch(
            "1..=3 matches 1, 2 or 3 times",
            many::<_, char>(1..=3, 'a'),
            &[
                ("", None, ""), //
                ("abcd", Some("a"), "bcd"),
                ("zzz", None, "zzz"),
                ("zaa", None, "zaa"),
                ("aaaaa", Some("aaa"), "aa"),
                ("aa|aaa", Some("aa"), "|aaa"),
            ],
        );

        test_lexer_batch(
            ".. matches any number of times",
            many::<_, char>(.., 'a'),
            &[
                ("", Some(""), ""), //
                ("abcd", Some("a"), "bcd"),
                ("zzz", Some(""), "zzz"),
                ("zaa", Some(""), "zaa"),
                ("aaaaa", Some("aaaaa"), ""),
                ("aa|aaa", Some("aa"), "|aaa"),
            ],
        );

        test_lexer_batch(
            "3..5 matches 3 or 4 times",
            many::<_, char>(3..5, 'a'),
            &[
                ("", None, ""), //
                ("abcd", None, "abcd"),
                ("zzz", None, "zzz"),
                ("zaa", None, "zaa"),
                ("aaaaa", Some("aaaa"), "a"),
                ("aa|aaa", None, "aa|aaa"),
                ("a|aaaa", None, "a|aaaa"),
                ("aaa|aa", Some("aaa"), "|aa"),
            ],
        );
    }
}
