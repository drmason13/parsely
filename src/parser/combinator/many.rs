use std::{
    fmt,
    ops::{Bound, RangeBounds},
};

use crate::{Parse, ParseError, ParseResult};

/// This parser is returned by [`many()`]. See it's documentation for more details.
pub struct Many<L: Parse> {
    /// The parser to be repeated.
    parser: L,

    /// The minimum number of times the parser must match for the parse to succeed.
    ///
    /// If the parser matches fewer than min times, the overall parse fails, and no input is consumed.
    min: usize,

    /// The maximum number of times the parser will attempt to match.
    ///
    /// The parser will never match more than max times, because it doesn't try to.
    ///
    /// To enforce that input is fully consumed after parsing, see [`crate::parsers::end()`]
    max: usize,
}

impl<L: Parse> Parse for Many<L> {
    fn parse<'i>(&mut self, input: &'i str) -> ParseResult<'i> {
        let mut count = 0;
        let mut offset = 0;
        let mut working_input = input;

        while count < self.max {
            if let Ok((output, remaining)) = self.parser.parse(working_input) {
                count += 1;
                offset += output.len();
                working_input = remaining;
            } else {
                break;
            }
        }

        if count < self.min {
            Err(ParseError::NoMatch)
        } else {
            Ok((&input[..offset], &input[offset..]))
        }
    }
}

/// Creates a parser that applies a given parser multiple times.
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
/// use parsely::{digit, Parse, ParseError};
/// use parsely::parser::combinator::many;
///
/// // these are all equivalent
/// let mut zero_or_more_digits = many(.., digit());
/// let mut zero_or_more_digits = many(0.., digit());
/// let mut zero_or_more_digits = many(0..usize::MAX, digit());
///
/// let (output, remaining) = zero_or_more_digits.parse("123")?;
/// assert_eq!(output, "123");
/// assert_eq!(remaining, "");
///
/// let (output, remaining) = zero_or_more_digits.parse("abc")?;
/// assert_eq!(output, "");
/// assert_eq!(remaining, "abc");
///
/// let mut one_or_more_digits = many(1.., digit());
///
/// let result = one_or_more_digits.parse("abc");
/// assert_eq!(result, Err(ParseError::NoMatch));
/// # Ok::<(), ParseError>(())
/// ```
///
/// Chain with [`Parse::many()`]:
///
/// ```
/// use parsely::{digit, Parse, ParseError};
///
/// let mut zero_or_more_digits = digit().many(0..);
///
/// # let (output, remaining) = zero_or_more_digits.parse("123")?;
/// # assert_eq!(output, "123");
/// # assert_eq!(remaining, "");
/// #
/// # let (output, remaining) = zero_or_more_digits.parse("abc")?;
/// # assert_eq!(output, "");
/// # assert_eq!(remaining, "abc");
/// # Ok::<(), ParseError>(())
/// ```
///
/// Min and Max:
///
/// ```
/// use parsely::{digit, Parse, ParseError};
///
/// let mut three_or_four_digits = digit().many(3..=4);
///
/// let (output, remaining) = three_or_four_digits.parse("123")?;
/// assert_eq!(output, "123");
/// assert_eq!(remaining, "");
///
/// let result = three_or_four_digits.parse("12");
/// assert_eq!(result, Err(ParseError::NoMatch));
///
/// let (output, remaining) = three_or_four_digits.parse("12345")?;
/// assert_eq!(output, "1234");
/// assert_eq!(remaining, "5");
/// # Ok::<(), ParseError>(())
/// ```
pub fn many<L: Parse>(range: impl RangeBounds<usize>, parser: L) -> Many<L> {
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

    Many { parser, min, max }
}

pub fn count<L: Parse>(count: usize, parser: L) -> Many<L> {
    Many {
        parser,
        min: count,
        max: count,
    }
}

impl<L: Parse> fmt::Display for Many<L>
where
    L: fmt::Display,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.max == usize::MAX {
            write!(f, "many({}.., {})", self.min, self.parser)
        } else {
            write!(f, "many({}..={}, {})", self.min, self.max, self.parser)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::char;
    use crate::test_utils::*;

    #[test]
    fn min_and_max() {
        test_parser_batch(
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

        test_parser_batch(
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

        test_parser_batch(
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