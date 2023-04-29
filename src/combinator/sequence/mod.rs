//! These combinators are used to parse sequences:
//!
//! * [`many()`] - match multiple times
//! * [`count()`] - match exactly n times
//! * [`.many().delimiter(lexer)`](many::Many::delimiter) - match multiple times, separated by something
//!
//! You might not need a sequence combinator. To match something and then another thing, see the humble [`then()`](crate::combinator::then()).
//!
//! When **parsing** a sequence, the output type is wrapped in a `Vec<T>` to store every match.
//!
//! Tip: Prefer using [`optional()`](crate::combinator::optional()) over `.many(0..=1)`. The former will output `Option<T>`, the latter will output `Vec<T>`.
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
//! [^max]: open-ended ranges limit themselves to matching `isize::MAX / 2` times, which for all practical purposes is any number of times.
//!
//! # Panics
//!
//! If a *minimum* that is greater than isize::MAX is given, then the internal `Vec` used to store the parser output will panic with `capacity overflow`:
//!
//! ```should_panic
//! # use parsely::{int, Parse};
//! let panic_parser = int::<u32>().many(usize::MAX..).parse("");  // this code will panic!
//! ```
//!
//! ```text
//! thread 'main' panicked at 'capacity overflow', library/alloc/src/raw_vec.rs:518:5
//! ```
mod delimited;
mod many;

pub use delimited::{delimited, Delimited};
pub use many::{count, many, Many};
