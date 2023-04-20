#![deny(rustdoc::broken_intra_doc_links)]

//!
//! Parsely is a parser combinator library for Rust with the following aims
//!
//! * Simple to use
//! * Well documented built-in parsers
//!
//! While it doesn't prioritise speed, it will often be "fast enough" for a projects that do a little bit of parsing here and there.
//!
//! If parsing speed is important to your application's performance (for example a compiler) then this library isn't meant for you.
//!
//! Take a look at the [`Parse`] trait and the built in [`parsers`] and [`combinators`].

mod parse;
pub mod parser;

pub use parse::{Parse, ParseError, ParseResult};
pub use parser::*;

#[doc(hidden)]
#[cfg(test)]
pub(crate) mod test_utils;
