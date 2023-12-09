use std::marker::PhantomData;
use std::{fmt, ops::RangeBounds};

use crate::{Lex, LexResult, Parse, ParseResult};

use super::delimited::Delimited;
use super::{min_max_from_bounds, MAX_LIMIT};

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

impl<P, C> Parse for Many<P, C>
where
    P: Parse,
    C: Default + Extend<<P as Parse>::Output>,
{
    type Output = C;

    fn parse<'i>(&self, input: &'i str) -> ParseResult<'i, Self::Output> {
        let mut count = 0;
        let mut offset = 0;
        let mut working_input = input;

        let mut outputs = C::default();

        while count < self.max {
            if let Ok((output, remaining)) = self.item.parse(working_input) {
                count += 1;
                offset = input.len() - remaining.len();
                outputs.extend(Some(output));
                working_input = remaining;
            } else {
                break;
            }
        }

        if count < self.min {
            Err(crate::Error::NoMatch)
        } else {
            Ok((outputs, &input[offset..]))
        }
    }
}

impl<L: Lex, C> Lex for Many<L, C> {
    fn lex<'i>(&self, input: &'i str) -> LexResult<'i> {
        let mut count = 0;
        let mut offset = 0;
        let mut working_input = input;

        while count < self.max {
            if let Ok((matched, remaining)) = self.item.lex(working_input) {
                count += 1;
                offset += matched.len();
                working_input = remaining;
            } else {
                break;
            }
        }

        if count < self.min {
            Err(crate::Error::NoMatch)
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
/// Basic usage:
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
/// assert_eq!(result, Err(parsely::Error::NoMatch));
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
/// assert_eq!(result, Err(parsely::Error::NoMatch));
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

impl<T, C> Many<T, C> {
    /// Creates a new parser that matches the same number of times, but expects the input to be separated by `delimiter`.
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
    /// let csv_parser = int::<u8>().many(1..).delimiter(char(','));
    ///
    /// let (output, remaining) = csv_parser.parse("1,2,3")?;
    /// assert_eq!(output, vec![1, 2, 3]);
    /// assert_eq!(remaining, "");
    /// # Ok::<(), parsely::Error>(())
    /// ```
    pub fn delimiter<L: Lex>(self, delimiter: L) -> Delimited<L, T, C> {
        let Many {
            min,
            max,
            item,
            collection: _,
        } = self;

        Delimited::new(min, max, item, delimiter)
    }
}

impl<T, O> Many<T, Vec<O>> {
    /// Adapts this [`Many`] parser to use a new collection instead of the default of `Vec<T>`.
    /// This method is analagous to [`Iterator::collect`].
    ///
    /// The new collection type must implement [`Extend`]. This trait is implemented for most [`std::collections`] types.
    ///
    /// Specify the collection type to use with a turbofish. Rust is often not able to infer the type you want to collect into.
    ///
    /// # Examples
    ///
    /// Basic usage:
    /// ```
    /// use std::collections::LinkedList;
    /// use parsely::{digit, char, Lex, Parse};
    ///
    /// let integers = digit().try_map(str::parse::<u8>).many(1..).collect::<LinkedList<u8>>();
    ///
    /// let (output, remaining) = integers.parse("123")?;
    /// assert_eq!(output, {
    ///     let mut linked_list = LinkedList::new();
    ///     linked_list.push_back(1);
    ///     linked_list.push_back(2);
    ///     linked_list.push_back(3);
    ///     linked_list
    /// });
    /// # Ok::<(), parsely::Error>(())
    /// ```
    ///
    /// Count to a HashMap during parsing:
    /// ```
    /// use std::collections::HashMap;
    /// use parsely::{any, char, int, Lex, Parse};
    ///
    /// let integers = any().map(str::to_string).then_skip(char(':')).then(int::<u8>()).many(1..).delimiter(char(',')).collect::<HashMap<String, u8>>();
    ///
    /// let (output, remaining) = integers.parse("a:1,b:2,c:3")?;
    /// assert_eq!(output, {
    ///     let mut map = HashMap::new();
    ///     map.insert("a".to_string(), 1);
    ///     map.insert("b".to_string(), 2);
    ///     map.insert("c".to_string(), 3);
    ///     map
    /// });
    /// # Ok::<(), parsely::Error>(())
    pub fn collect<C>(self) -> Many<T, C>
    where
        Self: Sized,
        C: Extend<O>,
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
            collection: PhantomData::<C>,
        }
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
    use crate::char;
    use crate::test_utils::*;

    #[derive(PartialEq, Debug, Clone)]
    struct A;
    impl FromStr for A {
        type Err = crate::Error;

        fn from_str(s: &str) -> Result<Self, Self::Err> {
            if s == "a" {
                Ok(A)
            } else {
                Err(crate::Error::NoMatch)
            }
        }
    }

    #[test]
    fn min_and_max_parse() {
        let a_parser = || char('a').try_map(A::from_str);

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
            many::<_, char>(1..=3, char('a')),
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
            many::<_, char>(.., char('a')),
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
            many::<_, char>(3..5, char('a')),
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
