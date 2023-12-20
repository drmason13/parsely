//! The [`Many`] combinator is used to parse sequences:
//!
//! * [`many()`] - match multiple times
//! * [`count()`] - match exactly n times
//!
//! Many has methods to adapt its behaviour:
//!
//! * [`.many().delimiter(lexer)`](many::Many::delimiter) - match multiple times, separated by something
//! * [`.many().or_until(lexer)`](many::Many::or_until) - stop early if a lexer matches the remaining input
//! * [`all()`](all::All) - match multiple times and expect End of Input afterwards or fail
//!
//! You might not need a sequence combinator. To match something and then another thing, see the humble [`then()`](crate::combinator::then()).
//!
//! When [**parsing**](crate::parse::Parse::parse) a sequence, the output type is wrapped in a [`Vec<T>`] to store every match.
//!
//! Tip: Prefer using [`optional()`](crate::combinator::optional()) over `.many(0..=1)`. The former will output [`Option<T>`], the latter will output [`Vec<T>`].
//!
//! ## Many
//!
//! [`many()`] is the most important sequence combinator.
//!
//! It can be used to lex multiple times, turning a lexer that consumes one character such as `digit()` into a lexer that consumes multiple characters:
//! ```
//! # use parsely::{digit, Lex};
//! digit().many(1..);
//! ```
//!
//! Parsers can use many, and their outputs are collected into a `Vec`:
//!
//! ```
//! # use parsely::{char, int, Lex, Parse};
//! let numbers_parser = int::<u32>().then_skip(char(',').optional()).many(1..);
//!
//! let (output, _) = numbers_parser.parse("123,456,789")?;
//! assert_eq!(output, vec![123, 456, 789]);
//!
//! # Ok::<(), parsely::Error>(())
//! ```
//!
//! The range argument to [`many()`] declares how many times the inner item must match.
//!
//! If the inner item does not match enough times then an [`Error`](crate::Error) is raised.
//!
//! If it could match more times, there's no error and no extra input is consumed.
//!
//! | range used | meaning                         |
//! |------------|---------------------------------|
//! | ..         | match any number of times[^max] |
//! | 1..        | match 1 or more times           |
//! | 0..        | match 0 or more times           |
//! | ..3        | match 0, 1, or 2 times          |
//! | ..n        | match 0 to n-1 times            |
//! | ..=3       | match 0, 1, 2 or 3 times        |
//! | ..=n       | match 0 to n times              |
//! | 3..=5      | match 3, 4 or 5 times           |
//! | a..=b      | match a to b times              |
//! | b..a       | if b > a: cannot match!         |
//!
//! This reflects the way [`std::ops::Range`] works with inclusive and exclusive bounds.
//!
//! [^max]: open-ended ranges limit themselves to matching `isize::MAX / 2` times, which for most purposes is more than plenty!
mod all;
mod delimited;
mod many;
mod or_until;

use std::ops::{Bound, RangeBounds};

pub use all::{all, All};
pub use delimited::{delimited, Delimited};
pub(crate) use many::LexMany;
pub use many::{count, many, Many};
pub use or_until::{or_until, OrUntil};

/// The maximum number of times to attempt to match a repeated parser and the implicit maximum for an open range.
pub(crate) const MAX_LIMIT: usize = (isize::MAX / 2) as usize;

pub(crate) fn min_max_from_bounds(range: impl RangeBounds<usize>) -> (usize, usize) {
    let min = match range.start_bound() {
        Bound::Included(&n) => n,
        Bound::Unbounded => 0,

        // start bounds cannot be excluded
        Bound::Excluded(_) => unreachable!(),
    };

    let max = match range.end_bound() {
        Bound::Included(&n) => n,
        Bound::Excluded(&n) => n.saturating_sub(1),
        Bound::Unbounded => MAX_LIMIT,
    };

    (min, max)
}

/// The sequence traits abstract how parsely sequence combinators repeatedly apply a lexer or parser to an input
///
/// These traits should not need to be implemented manually, prefer to use existing combinators such as [`many()`](crate::combinator::many)
pub mod traits {
    use std::ops::ControlFlow;

    use crate::{Error, Lex, Parse};

    use super::Delimited;

    /// Describes how a sequence combinator behaves while processing input
    pub trait Sequence: Collect {
        /// The sequencer continues to process input **while this returns true**
        fn while_condition(&self, input: &str, count: usize) -> bool;

        /// The sequencer returns an error instead of succeeding if this returns true
        ///
        /// It is called after all processable input has been processed
        fn error_condition(&self, input: &str, count: usize) -> bool;
    }

    /// Adapts this [`sequence`](self) parser to use a new collection instead of the default of `Vec<T>`.
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
    /// use parsely::{digit, char, sequence_traits::*, Lex, Parse};
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
    /// use parsely::{any, char, int, sequence_traits::*, Lex, Parse};
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
    pub trait Collect {
        /// The type returned when calling collect, where C is the new Collection type to use
        ///
        /// Almost always `Self<C>` but we have to use an associated type to describe that
        type Output<C>;

        /// Change the collection used by a [sequencer](Sequence) to C
        fn collect<C1>(self) -> Self::Output<C1>
        where
            Self: Sized;
        // Self::Output<C1>: ParseSequence<C1>;
    }

    /// All sequence combinators impl both [`LexSequence`] and [`ParseSequence`]
    pub trait LexSequence: Sequence {
        /// The [`Lexer`](crate::Lex) to apply repeatedly
        type Lexer: Lex;

        /// progress through one iteration of lexing
        fn lex_one<'i>(
            &self,
            input: &'i str,
            working_input: &mut &'i str,
            count: &mut usize,
            offset: &mut usize,
            error: &mut Option<Error<'i>>,
        ) -> ControlFlow<(), &'i str>;

        /// Creates a new lexer that matches the same sequence, but expects the input to be separated by `delimiter`.
        ///
        /// A trailing match is optional, so this is suitable for lexing separated lists.
        ///
        /// See also [ParseSequence::delimited]
        fn delimiter<L: Lex>(self, delimiter: L) -> Delimited<L, Self, ()>
        where
            Self: Sized,
        {
            Delimited::new(self, delimiter)
        }
    }

    /// All sequence combinators impl both [`LexSequence`] and [`ParseSequence`]
    pub trait ParseSequence<C>: Sequence
    where
        C: Extend<<Self::Parser as Parse>::Output>,
    {
        /// The [`Parser`](crate::Parse) to apply repeatedly
        type Parser: Parse;

        /// progress through one iteration of parsing
        fn parse_one<'i>(
            &self,
            input: &'i str,
            working_input: &mut &'i str,
            count: &mut usize,
            offset: &mut usize,
            error: &mut Option<Error<'i>>,
            outputs: &mut C,
        ) -> ControlFlow<(), &'i str>;

        /// Creates a new parser that matches the same sequence, but expects the input to be separated by `delimiter`.
        ///
        /// A trailing match is optional, so this is suitable for parsing separated lists.
        ///
        /// # Examples
        ///
        /// Basic usage:
        ///
        /// ```
        /// use parsely::{char, int, Parse, sequence_traits::*};
        ///
        /// let csv_parser = int::<u8>().all(1).delimiter(char(','));
        ///
        /// let (output, remaining) = csv_parser.parse("1,2,3").expect("ok okay geez");
        /// assert_eq!(output, vec![1, 2, 3]);
        /// assert_eq!(remaining, "");
        ///
        /// let result = csv_parser.parse("1,2,3foo");
        /// assert_eq!(result.unwrap_err().remaining, "foo");
        /// # Ok::<(), parsely::Error>(())
        /// ```
        fn delimiter<L: Lex>(self, delimiter: L) -> Delimited<L, Self, C>
        where
            Self: Sized,
        {
            Delimited::new(self, delimiter)
        }
    }
}
