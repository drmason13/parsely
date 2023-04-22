use std::{
    fmt,
    ops::{Bound, RangeBounds},
};

use crate::{Lex, LexResult, Parse, ParseResult};

/// This combinator is returned by [`many()`]. See it's documentation for more details.
pub struct Many<T> {
    /// The parser to be repeated.
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
}

impl<P: Parse> Parse for Many<P> {
    type Output = Vec<<P as Parse>::Output>;

    fn parse<'i>(&mut self, input: &'i str) -> ParseResult<'i, Self::Output> {
        let mut count = 0;
        let mut offset = 0;
        let mut working_input = input;

        let mut outputs = Vec::with_capacity(self.max);

        while count < self.max {
            if let Ok((output, remaining)) = self.item.parse(working_input) {
                count += 1;
                offset = input.len() - remaining.len();
                outputs.push(output);
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

impl<L: Lex> Lex for Many<L> {
    fn lex<'i>(&mut self, input: &'i str) -> LexResult<'i> {
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
/// # Examples
///
/// Basic usage:
///
/// ```
/// use parsely::{digit, Lex};
/// use parsely::combinator::many;
///
/// // these are all equivalent
/// let mut zero_or_more_digits = many(.., digit());
/// let mut zero_or_more_digits = many(0.., digit());
/// let mut zero_or_more_digits = many(0..usize::MAX, digit());
///
/// let (output, remaining) = zero_or_more_digits.lex("123")?;
/// assert_eq!(output, "123");
/// assert_eq!(remaining, "");
///
/// let (output, remaining) = zero_or_more_digits.lex("abc")?;
/// assert_eq!(output, "");
/// assert_eq!(remaining, "abc");
///
/// let mut one_or_more_digits = many(1.., digit());
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
/// let mut zero_or_more_digits = digit().many(0..);
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
/// let mut three_or_four_digits = digit().many(3..=4);
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
pub fn many<T>(range: impl RangeBounds<usize>, item: T) -> Many<T> {
    let min = match range.start_bound() {
        Bound::Included(&n) => n,
        Bound::Unbounded => 0,

        // start bounds cannot be excluded
        Bound::Excluded(_) => unreachable!(),
    };

    let max = match range.end_bound() {
        Bound::Included(&n) => n,
        Bound::Excluded(&n) => n.saturating_sub(1),
        Bound::Unbounded => usize::MAX,
    };

    Many { item, min, max }
}

pub fn count<T>(count: usize, item: T) -> Many<T> {
    Many {
        item,
        min: count,
        max: count,
    }
}

impl<T> fmt::Debug for Many<T>
where
    T: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.max == usize::MAX {
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
        let a_parser = char('a').map(A::from_str);

        test_parser_batch(
            "1..=3 matches 1, 2 or 3 times",
            many(1..=3, a_parser.clone()),
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
            many(.., a_parser.clone()),
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
            many(3..5, a_parser),
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
            many(1..=3, char('a')),
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
            many(.., char('a')),
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
            many(3..5, char('a')),
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
