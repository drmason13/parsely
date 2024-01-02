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
//! All lexers can be run using their [`lex()`](crate::Lex::lex()) method from the [`Lex`](crate::Lex) trait.
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
//! ##How do I make a parser from my lexer?
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
    alpha, alphanum, ascii_alpha, ascii_alphanum, ch, ch_if, lowercase, none_of, one_of, uppercase,
    ws, Char, WhiteSpace,
};
pub use self::end::{end, End};
pub use self::number::{digit, hex, non_zero_digit, Digit};
pub use self::take::{take, take_while, Take, TakeWhile};
pub use self::token::{itoken, token, Token};
pub use self::until::{until, Until};

/// Used as a generic parameter to combinators that can either [`Parse`] or [`Lex`] and need disambiguating
///
/// [`Parse`]: crate::Parse
/// [`Lex`]: crate::Lex
pub struct Lexing;
