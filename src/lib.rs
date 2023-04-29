#![deny(rustdoc::broken_intra_doc_links)]
#![deny(missing_docs)]

//! Parsely is a simple string parsing library for Rust with the following aims
//!
//! * Simple to use
//! * Well documented
//!
//! While it doesn't prioritise speed, it will often be "fast enough" for a projects that do a little bit of parsing here and there.
//!
//! Parsely provides combinators for you to build up complex parsers from simple reusable pieces.
//!
//! What makes Parsely different from other (excellent) parser combinator libraries?
//!
//! - the limitation of UTF-8 [`&str`](prim@str) input and the speed and simplicity this affords.
//! - the fundamental split between **lexing** (splitting strings into smaller parts) and **parsing** (converting string parts into other types).
//!
//! Take a look at the [`Lex`] and [`Parse`] traits and the module level documentation: [`lexer`], [`parser`] and [`combinator`].

mod error;
pub use error::Error;

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
