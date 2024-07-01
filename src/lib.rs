#![deny(rustdoc::broken_intra_doc_links)]
#![deny(missing_docs)]
#![doc(test(attr(deny(unused_imports))))]

//! # Parsely 🌿
//!
//! Parsely is a simple string parsing library for Rust with the following aims
//!
//! * Excel when used to `impl FromStr` for your types
//! * Simple to use
//! * Well documented
//!
//! # Example
//! ```
#![doc = include_str!("../examples/canonical_example.rs")]
//! ```
//!
//! Parsely provides combinators for you to build up complex parsers from simple reusable pieces.
//!
//! What makes Parsely different from other (excellent) parser combinator libraries?
//!
//! * the limitation of UTF-8 [`&str`](prim@str) input and the speed and simplicity this affords.
//! * the API split between *lexing*[^terminology] (splitting strings into smaller parts) and *parsing*[^terminology] (converting string parts into other types).
//! * no macros in the public API.
//!
//! Take a look at the [`Lex`] and [`Parse`] traits and the module level documentation: [`lexer`], [`parser`] and [`combinator`].
//!
//! ## Comparison to other Rust parsing libraries:
//!
//! | crate   | style                    | notes |
//! |---------|--------------------------|-------|
//! | nom     | Parser Combinators       | Excellent at parsing bytes (and strings). Generic over input and error types and streaming support. Mature and battle tested. Can be quite complex when error handling. [Lots of parser and combinators](https://github.com/rust-bakery/nom/blob/main/doc/choosing_a_combinator.md) to choose from. |
//! | yap     | [`Iterator`]-like design | Generic over input type. Simple for those unfamiliar with parser combinators. Tends to be verbose. Well documented |
//! | combine | Parser Combinators       | Trait based approach. Generic over input type and streaming support - including `Read` instances. Zero copy parsing. |
//! | chumsky | Parser Combinators       | Exceptional error handling and recovery. Prioritises error handling and recovery over speed. Generic over input and error types. |
//! | lalrpop | Parser Generator         | Useful error messages. LR or LALR parsers. Requires a build.rs script. |
//! | logos   | Lexer                    | Exceptionally fast at producing tokens from string input. Proc macro based. |
//! | parsely | Parser combinators       | &str input only. No macros. Simple and intuitive. Suitable for parsing short simple input |
//! | pest    | PEG parser generator     | Proc macro based. Requires writing a grammar file to describe your parsing. More suited to describing languages |
//!
//! [`Iterator`]: std::iter::Iterator
//!
//! [^terminology]: These are the terms as used and understood in this library.
//! I believe what we call "lexing", many would call "tokenising"; and what we call "parsing" many would call "lexing".
//! Parsely doesn't parse into a tree-like structure at any point, that would be up to the user to do.
//! If our inexact usage of these terms irks you, then I recommend a parser combinator library intended for parsing programming languages such as [Chumsky](https://docs.rs/chumsky/latest/chumsky/).
//!
//! # Rough edges
//!
//! Parsely has some rough edges I've gotten used to, I'm going to try and keep track of them here:
//!
//! 1. The obvious approach to implementing [`FromStr`] causes an error like the following:
//!
//! ```console
//! error[E0521]: borrowed data escapes outside of associated function
//! --> src\expr.rs:212:25
//!  |
//! 209 |     fn from_str(s: &str) -> Result<Self, Self::Err> {
//!  |                 -  - let's call the lifetime of this reference `'1`
//!  |                 |
//!  |                 `s` is a reference that is only valid in the associated function body
//! ...
//! 212 |         let (expr, _) = expr.then_skip(end()).parse(s)?;
//!  |                         ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
//!  |                         |
//!  |                         `s` escapes the associated function body here
//!  |                         argument requires that `'1` must outlive `'static`
//!
//! For more information about this error, try `rustc --explain E0521`.
//! error: could not compile `popvars` (lib) due to previous error
//! warning: build failed, waiting for other jobs to finish...
//! error: could not compile `popvars` (lib test) due to previous error
//! ```
//!
//! The solution is to use [`ErrorOwned`] instead of [`Error`], which has no lifetime parameter. `?` will convert between the two just fine.
//!
//! [`FromStr`]: std::str::FromStr
//! [`impl FromStr`]: std::str::FromStr

/// Whether the item is being used to do [`Parsing`](parser::Parsing) or [`Lexing`](lexer::Lexing).
///
/// [`Behavior`] is a bound for generic parameters of flexible items that implement both [`Parse`] and [`Lex`].
/// The generic Parameter is a more ergonomic way of disambiguating the usage.
///
/// Note: This trait is [sealed](https://predr.ag/blog/definitive-guide-to-sealed-traits-in-rust/#sealing-traits-with-a-supertrait).
pub trait Behavior: private::Sealed {}

impl Behavior for parser::Parsing {}
impl private::Sealed for parser::Parsing {}

impl Behavior for lexer::Lexing {}
impl private::Sealed for lexer::Lexing {}

/// The built in combinators provided by parsely
pub mod combinator {
    mod crawl;
    mod map;
    // pub mod map_error;
    mod optional;
    mod or;
    mod pad;
    pub mod sequence {
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
        //! # use parsely::{int, Lex, Parse};
        //! let numbers_parser = int::<u32>().then_skip(','.optional()).many(1..);
        //!
        //! let (output, _) = numbers_parser.parse("123,456,789")?;
        //! assert_eq!(output, vec![123, 456, 789]);
        //! #
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
        //!
        //! ## A more hands-on example:
        //!
        //! This example is included from `examples/sequence.rs`.
        //!
        //! ```
        #![doc = include_str!("../examples/sequence.rs")]
        //! ```
        mod all;
        mod delimited;
        mod many;
        mod or_until;
        pub mod traits;

        pub use all::{all, All};
        pub use delimited::{delimited, Delimited};
        pub(crate) use many::LexMany;
        pub use many::{count, many, Many};
        pub use or_until::{or_until, OrUntil};

        use std::ops::{Bound, RangeBounds};

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
    }
    pub mod skip;
    mod then;

    #[doc(inline)]
    pub use self::crawl::crawl;
    pub use self::crawl::Crawl;
    #[doc(inline)]
    pub use self::map::{map, try_map};
    pub use self::map::{Map, TryMap};
    #[doc(inline)]
    pub use self::optional::optional;
    pub use self::optional::Optional;
    #[doc(inline)]
    pub use self::or::or;
    pub use self::or::Or;
    #[doc(inline)]
    pub use self::pad::pad;
    pub use self::pad::Pad;
    #[doc(inline)]
    pub use self::sequence::traits::Collect;
    #[doc(inline)]
    pub use self::sequence::{all, count, delimited, many};
    pub use self::sequence::{All, Delimited, Many};
    #[doc(inline)]
    pub use self::skip::{skip_then, then_skip};
    pub use self::skip::{SkipThen, ThenSkip};
    #[doc(inline)]
    pub use self::then::then;
    pub use self::then::Then;
}

mod error;
pub use error::{result_ext, Error, ErrorOwned, ErrorReason};

mod lex;
pub use lex::{Lex, LexResult};

pub mod lexer {
    //! The built in lexers provided by parsely
    //!
    //! You can create your own lexers by combining these built-in lexers as you need. You can think of these as "lexer primitives".
    //!
    //! Additionally, you can write a function that takes an input, and returns a tuple `(matched, remaining)` that borrows from the input to create a completely new "lexer primitive".
    //!
    //! If you think a useful lexer primitive is missing, please raise an issue. We might not be able to include all of them but it should be rare that you *need* a custom lexer primitive.
    //!
    //! # Examples:
    //!
    //! All lexers can be run using their [`lex()`](crate::Lex::lex()) method from the [`Lex`] trait.
    //!
    //! ```
    //! use parsely::{token, Lex};
    //!
    //! // first get your lexer
    //! let my_token = token("match this");
    //!
    //! // then use it to lex an input string - note the `?` because lex() returns an error if the input doesn't match.
    //! let (matched, remaining) = my_token.lex("match this, but not this")?;
    //!
    //! // lexing split the string into two - first the part that matched, then the remaining part
    //! assert_eq!(matched, "match this");
    //! assert_eq!(remaining, ", but not this");
    //! # Ok::<(), parsely::Error>(())
    //! ```
    //!
    //! Combine two lexers with [`then`](crate::combinator::then):
    //!
    //! ```
    //! use parsely::{token, Lex};
    //!
    //! let my_token = token("match this");
    //!
    //! let my_other_token = token("then this");
    //!
    //! // here we combine our two lexers using then()
    //! let combined = my_token.then(my_other_token);
    //!
    //! // this won't match because ", " isn't our other token
    //! let result = combined.lex("match this, then this");
    //!
    //! // let's fix that
    //! # let my_token = token("match this");
    //! # let my_other_token = token("then this");
    //! // we don't need to name every part of our lexer, here we'll simply add a token() call in between our 2 lexers.
    //! let combined = my_token.then(token(", ")).then(my_other_token);
    //!
    //! // now it works
    //! let (matched, remaining) = combined.lex("match this, then this")?;
    //! assert_eq!(matched, "match this, then this");
    //! assert_eq!(remaining, "");
    //! # Ok::<(), parsely::Error>(())
    //! ```
    //!
    //! There are many other methods avilable to combine lexers in useful ways including
    //! [`or()`](crate::Lex::or),
    //! [`skip_then()`](crate::Lex::skip_then()),
    //! and [`then_skip()`](crate::Lex::then_skip).
    //!
    //! They are all methods of the [Lex trait](crate::Lex).
    //!
    //! ## How do I make a parser from my lexer?
    //!
    //! Take a look at the [parser module](crate::parser) which has examples of building parsers out of custom lexers, built-in lexers and combinations there of!
    //!
    //! TL;DR: use [`map()`]
    //!
    //! [`Parse`]: crate::Parse
    //! [`Lex`]: crate::Lex
    //! [`lexer`]: crate::lexer
    //! [`map()`]: crate::Lex::map

    mod any;
    mod char;
    mod end;
    mod number;
    mod take;
    mod token;
    mod until;

    pub use self::any::{any, Any};
    pub use self::char::{
        alpha, alphanum, ascii_alpha, ascii_alphanum, ch, ch_if, lowercase, none_of, one_of,
        uppercase, ws, Char, WhiteSpace,
    };
    pub use self::end::{end, End};
    pub use self::number::{digit, hex, non_zero_digit, Digit};
    pub use self::take::{take, take_while, Take, TakeWhile};
    pub use self::token::{itoken, token, Token};
    pub use self::until::{until, Until};

    /// Used as a generic parameter to items that can either [`Parse`] or [`Lex`] and need disambiguating
    ///
    /// [`Parse`]: crate::Parse
    /// [`Lex`]: crate::Lex
    pub struct Lexing;

    /// Case Sensitivity is a sealed trait for [`CaseSensitive`] and [`CaseInsensitive`] used by [`token()`] and [`ch()`]
    ///
    /// Note: This trait is [sealed](https://predr.ag/blog/definitive-guide-to-sealed-traits-in-rust/#sealing-traits-with-a-supertrait).
    pub trait CaseSensitivity: crate::private::Sealed {}

    /// Case Sensitive lexers match only if the input matches case exactly
    pub struct CaseSensitive;

    /// Case Insensitive lexers match regardless of the input case
    pub struct CaseInsensitive;

    impl CaseSensitivity for CaseSensitive {}
    impl crate::private::Sealed for CaseSensitive {}
    impl CaseSensitivity for CaseInsensitive {}
    impl crate::private::Sealed for CaseInsensitive {}
}
#[doc(inline)]
pub use lexer::{
    alpha, alphanum, any, ascii_alpha, ascii_alphanum, ch, ch_if, digit, end, hex, itoken,
    lowercase, non_zero_digit, none_of, one_of, take, take_while, token, until, uppercase, ws,
};

mod parse;
pub use parse::{Parse, ParseResult};

pub mod parser {
    //! The built in parsers provided by parsely
    //!
    //! These functions return a type implementing [`Parse`].
    //!
    //! # Examples
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
    //! # Ok::<(), parsely::Error>(())
    //! ```
    //!
    //! Custom types can be parsed using map and switch. Here's a snippet from the [json example]
    //!
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
    //! # Ok::<(), parsely::Error>(())
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

    pub use escape::{escape, escape_lex, EscapeSequence};
    pub use number::{float, int, number, uint};
    pub use switch::switch;

    /// Used as a generic parameter to items that can either [`Parse`] or [`Lex`] and need disambiguating
    ///
    /// [`Parse`]: crate::Parse
    /// [`Lex`]: crate::Lex
    pub struct Parsing;
}
#[doc(inline)]
pub use parser::{escape, escape_lex, float, int, number, switch, uint};

mod private {
    /// Sealed trait pattern: https://predr.ag/blog/definitive-guide-to-sealed-traits-in-rust/#sealing-traits-with-a-supertrait
    pub trait Sealed {}
}

#[doc(hidden)]
#[cfg(test)]
pub(crate) mod test_utils;

#[doc(hidden)]
#[cfg(test)]
mod test_automation {
    use crate::{error::result_ext::*, token, until, ws, Lex};

    #[test]
    fn sync_readme_example() -> Result<(), Box<dyn std::error::Error>> {
        let example_path = "examples/canonical_example.rs";
        let example = std::fs::read_to_string(example_path)?;

        let readme_path = "README.md";
        let readme = std::fs::read_to_string(readme_path)?;

        let fence = "```";

        let (start, remaining) = until("## Example")
            .then(
                token("## Example")
                    .then(ws().many(..))
                    .then(token(fence))
                    .then(token("rust"))
                    .then('\n'),
            )
            .lex(&readme)
            .own_err()?;

        let (_, end) = until(fence).lex(remaining).own_err()?;

        let output = {
            let mut s = start.to_string();
            s.push_str(&example);
            s.push_str(end);
            s
        };

        std::fs::write(readme_path, output)?;

        Ok(())
    }
}
