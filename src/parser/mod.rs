//! The built in parsers provided by parsely
//!
//! These functions return a type implementing [`Parse`].
//!
//! # Examples
//!
//! Basic usage:
//!
//! ```
//! use parsely::{Lex, Parse, uint};
//!
//! let id_parser = "#".skip_then(uint::<u32>());
//!
//! let (output, remaining) = id_parser.parse("#123abc")?;
//!
//! assert_eq!(output, 123);
//! assert_eq!(remaining, "abc");
//! # Ok::<(), parsely::InProgressError>(())
//! ```
//!
//! Custom types can be parsed using map and switch. Here's a snippet from the [json example]
//! ```
//! use parsely::{Lex, Parse, int, float};
//!
//! /// A float or integer
//! #[derive(Debug, PartialEq)]
//! pub struct Number(N);
//!
//! // This strategy is inspired by serde_json
//! #[derive(Debug, PartialEq)]
//! pub enum N {
//!     Int(i64),
//!     Float(f64),
//! }
//!
//! fn number() -> impl Parse<Output = Number> {
//!     (float::<f64>().map(|n| Number(N::Float(n)))).or(int::<i64>().map(|n| Number(N::Int(n))))
//! }
//!
//! fn bool() -> impl Parse<Output = bool> {
//!     "true".map(|_| true).or("false".map(|_| false))
//! }
//!
//! assert_eq!(number().parse("1")?.0, Number(N::Int(1)));
//! assert_eq!(number().parse("123.45")?.0, Number(N::Float(123.45)));
//!
//! assert_eq!(bool().parse(r"true")?.0, true);
//! assert_eq!(bool().parse(r"false")?.0, false);
//! # Ok::<(), parsely::InProgressError>(())
//! ```
//!
//! See also [`lexer`] for types implementing [`Lex`].
//!
//! [`Parse`]: crate::Parse
//! [`Lex`]: crate::Lex
//! [`lexer`]: crate::lexer
//! [json example]: https://github.com/drmason13/parsely/blob/main/examples/json.rs
mod escape;
mod number;
mod switch;

pub use self::number::{float, int, number, uint};
pub use self::switch::switch;
pub use escape::{escape, escape_lex, EscapeSequence};

/// Used as a generic parameter to combinators that can either [`Parse`] or [`Lex`] and need disambiguating
///
/// [`Parse`]: crate::Parse
/// [`Lex`]: crate::Lex
pub struct Parsing;
