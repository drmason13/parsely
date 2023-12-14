#![deny(rustdoc::broken_intra_doc_links)]
#![deny(missing_docs)]

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
//! The solution is to use [`ErrorOwned`] instead, which has no lifetime parameter. `?` will convert between the two just fine.
//!
//! [`FromStr`]: std::str::FromStr
//! [`impl FromStr`]: std::str::FromStr
mod error;
mod tokenize;
mod tokenize2;
pub use error::{result_ext, Error, ErrorOwned, ErrorReason};

mod lex;
pub mod lexer;

pub use lex::{Lex, LexResult};
pub use lexer::*;

mod parse;
pub mod parser;

pub use parse::{Parse, ParseResult};
pub use parser::*;

pub mod combinator;

#[doc(hidden)]
#[cfg(test)]
pub(crate) mod test_utils;

#[doc(hidden)]
#[cfg(test)]
mod test_automation {
    use crate::{char, error::result_ext::*, token, until, ws, Lex};

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
                    .then(char('\n')),
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
