//! # The Fundamental Law of Parsely Lexing
//!
//! The fundamental law of parsely lexing states that a [lexer] must return two contiguous str slices of its input.
//!
//! The important result of this law is
//!
//! > **No part of the input can be skipped while lexing**
//!
//! Many parsely combinators will skip or ignore[^ignored_vs_skippped] part of their input, but skipping *only* happens when they are parsing.
//!
//! # Example
//!
//! [`pad`](pad) *cannot* be used as a lexer because it would violate the fundamental law of parsely lexing:
//!
//! ```compile_fail
//! # use parsely::*;
//! #
//! let lexer = token("foo").pad()  // pad skips/ignores surrounding whitespace
//!                         .then(token("bar"));
//!
//! lexer.lex("foo   bar")?;
//! #
//! # Ok::<(), parsely::Error>(())
//! ```
//!
//! pad can be used as a *parser*:
//!
//! ```
//! # use parsely::*;
//! #
//! let parser = token("foo")
//!     .map(str::to_string)
//!     .pad()
//!     .then(token("bar").map(str::to_string));
//!
//! assert_eq!(
//!     parser.parse("foo   bar")?,
//!     (("foo".to_string(), "bar".to_string()), "")
//! );
//!
//! // if we want the string "foobar" we would need to do something inefficient to join two Strings
//! let parser = token("foo")
//!     .map(str::to_string)
//!     .pad()
//!     .then(token("bar").map(str::to_string))
//!     .map(|(a, b)| format!("{a}{b}"));
//!
//! assert_eq!(parser.parse("foo   bar")?, (("foobar".to_string()), ""));
//! #
//! # Ok::<(), parsely::Error>(())
//! ```
//!
//! As you can see, [`pad()`](pad) behaved differently depending on whether it was used to [`parse`](Parse) or [`lex`](Lex).
//!
//! [^ignored_vs_skippped]: By "ignored" we mean it is seen by the lexer, allowed to match, and included in the output.
//! By "skipped" we mean it is seen by the lexer, allowed to match, and discarded from the output. A parser never sees "skipped" input.
//!
//! When lexing, it must uphold the fundamental law of parsely lexing and preserve the whitespace.
//! It *ignores* it in that it won't prevent a match but it is included in the output.
//!
//! When parsing, it is free to actually *skip* the whitespace so it isn't included in the output.

use crate::{combinator::pad, lexer, Lex, Parse};
