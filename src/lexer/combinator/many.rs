use std::{
    fmt,
    ops::{Bound, RangeBounds},
};

use crate::{Lex, LexError, LexResult};

/// This lexer is returned by [`many()`]. See it's documentation for more details.
pub struct Many<L: Lex> {
    /// The lexer to be repeated.
    lexer: L,

    /// The minimum number of times the lexer must match for the lex to succeed.
    ///
    /// If the lexer matches fewer than min times, the overall lex fails, and no input is consumed.
    min: usize,

    /// The maximum number of times the lexer will attempt to match.
    ///
    /// The lexer will never match more than max times, because it doesn't try to.
    ///
    /// To enforce that input is fully consumed after parsing, see [`crate::lexers::end()`]
    max: usize,
}

impl<L: Lex> Lex for Many<L> {
    fn lex<'i>(&mut self, input: &'i str) -> LexResult<'i> {
        let mut count = 0;
        let mut offset = 0;
        let mut working_input = input;

        while count < self.max {
            if let Ok((output, remaining)) = self.lexer.lex(working_input) {
                count += 1;
                offset += output.len();
                working_input = remaining;
            } else {
                break;
            }
        }

        if count < self.min {
            Err(LexError::NoMatch)
        } else {
            Ok((&input[..offset], &input[offset..]))
        }
    }
}

/// Creates a lexer that applies a given lexer multiple times.
///
/// This function takes a Range-like argument as a succint description of start and end bounds.
///
/// The start bound becomes the minimum number of times the lexer must match to succeed.
///
/// The end bound becomes the maximum number of times the lexer will attempt to lex.
///
/// # Examples
///
/// Basic usage:
///
/// ```
/// use parsely::{digit, Lex, LexError};
/// use parsely::lexer::combinator::many;
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
/// assert_eq!(result, Err(LexError::NoMatch));
/// # Ok::<(), LexError>(())
/// ```
///
/// Chain with [`Lex::many()`]:
///
/// ```
/// use parsely::{digit, Lex, LexError};
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
/// # Ok::<(), LexError>(())
/// ```
///
/// Min and Max:
///
/// ```
/// use parsely::{digit, Lex, LexError};
///
/// let mut three_or_four_digits = digit().many(3..=4);
///
/// let (output, remaining) = three_or_four_digits.lex("123")?;
/// assert_eq!(output, "123");
/// assert_eq!(remaining, "");
///
/// let result = three_or_four_digits.lex("12");
/// assert_eq!(result, Err(LexError::NoMatch));
///
/// let (output, remaining) = three_or_four_digits.lex("12345")?;
/// assert_eq!(output, "1234");
/// assert_eq!(remaining, "5");
/// # Ok::<(), LexError>(())
/// ```
pub fn many<L: Lex>(range: impl RangeBounds<usize>, lexer: L) -> Many<L> {
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

    Many { lexer, min, max }
}

pub fn count<L: Lex>(count: usize, lexer: L) -> Many<L> {
    Many {
        lexer,
        min: count,
        max: count,
    }
}

impl<L: Lex> fmt::Display for Many<L>
where
    L: fmt::Display,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.max == usize::MAX {
            write!(f, "many({}.., {})", self.min, self.lexer)
        } else {
            write!(f, "many({}..={}, {})", self.min, self.max, self.lexer)
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
