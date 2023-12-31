//! Skip combinators are used to match without storing output.
//!
//! The complete set of valid interactions between [`Parse`], [`Lex`], [`skip_then()`] and [`then_skip()`]:
//!
//! | usage                    | description                                         |
//! |--------------------------|-----------------------------------------------------|
//! | parser.then_skip(lexer)  | parse and then ignore some of the remaining input   |
//! | lexer.then_skip(lexer)   | lex and then ignore some of the remaining input     |
//! | parser.skip_then(lexer)  | parse, but ignore the output; then then run a lexer |
//! | lexer.skip_then(lexer)   | lex, but ignore the matched part; then run a lexer  |
//! | parser.skip_then(parser) | parse, but ignore the output; then run a parser     |
//! | lexer.skip_then(parser)  | lex, but ignore the matched part; then run a parser |
//!
//! In practice, these two are commonly used:
//!
//! * `lexer.then_skip(parser)`
//! * `parser.then_skip(lexer)`
//!
//! Let's demonstrate `lexer.then_skip(parser)` with a simple hex color code parser
//!
//! ```
//! use parsely::{char, hex, Lex, Parse};
//!
//! // we want to parse input like #AABBCC into 3 u8 representing an RGB color
//! #[derive(Debug, PartialEq)]
//! struct Rgb {
//!     red: u8,
//!     green: u8,
//!     blue: u8,
//! }
//!
//! fn hex_u8() -> impl Parse<Output = u8> {
//!     hex()           // match a hex digit...
//!         .count(2)   // ...2 of them
//!         .try_map(|s| u8::from_str_radix(s, 16))  // and try to map that string (e.g. "AA") a u8!
//! }
//!
//! let hex_color =
//!     char('#')        // `#` is part of the input we want to match, but we don't want it when converting to u8
//!         .skip_then(  // so we'll skip it, and then run a parser that convders hex to u8
//!             hex_u8()
//!                 .then(hex_u8())
//!                 .then(hex_u8()) // we need 3 u8s, we could use .count(3) but that would allocate into a vec, which we'd only then have to unpack anyway
//!         );
//!
//! // let's test it
//!
//! assert_eq!(hex_color.parse("#AABBCC")?, (((170, 187), 204), ""));
//!
//! // oh right, we never mapped out u8s into an Rgb. That's just a .map() away:
//! let hex_rgb = hex_color.map(|((red, green), blue)| Rgb { red, green, blue });
//!
//! assert_eq!(hex_rgb.parse("#AABBCC")?, (Rgb { red: 170, green: 187, blue: 204 }, ""));
//! # Ok::<(), parsely::InProgressError>(())
//! ```

use crate::{result_ext::*, Lex, Parse};

/// This combinator is returned by [`then_skip()`]. See it's documentation for more details.
#[derive(Debug, Clone)]
pub struct ThenSkip<L, T> {
    lexer: L,
    item: T,
}

/// *After* running the item (parser or lexer), this combinator will run the given lexer and discard its output.
///
/// If the lexer fails, it is still a parse failure. Use `.optional()` if the input to be skipped isn't required.
///
/// This combinator can be chained using [`Parse::then_skip()`].
//
// TODO: what actually happens if you do a then_skip(lexer_a, lexer_b) ???
pub fn then_skip<L: Lex, T>(lexer: L, item: T) -> ThenSkip<L, T> {
    ThenSkip { lexer, item }
}

impl<L: Lex, T: Lex> Lex for ThenSkip<L, T> {
    fn lex<'i>(&self, input: &'i str) -> crate::LexResult<'i> {
        let (output, remaining) = self.item.lex(input).offset(input)?;
        let (_, remaining) = self.lexer.lex(remaining).offset(input)?;
        Ok((output, remaining))
    }
}

impl<L: Lex, T: Parse> Parse for ThenSkip<L, T> {
    type Output = <T as Parse>::Output;

    fn parse<'i>(&self, input: &'i str) -> crate::ParseResult<'i, Self::Output> {
        let (output, remaining) = self.item.parse(input).offset(input)?;
        let (_, remaining) = self.lexer.lex(remaining).offset(input)?;

        Ok((output, remaining))
    }
}

/// This combinator is returned by [`skip_then()`]. See it's documentation for more details.
#[derive(Debug, Clone)]
pub struct SkipThen<L, T> {
    lexer: L,
    item: T,
}

/// *Before* running the item (parser or lexer), this combinator will run the given lexer and discard its output.
///
/// If the lexer fails, it is still a parse failure. Use `.optional()` if the input to be skipped isn't required.
///
/// This combinator can be chained using [`Lex::skip_then()`].
pub fn skip_then<L: Lex, T>(lexer: L, item: T) -> SkipThen<L, T> {
    SkipThen { lexer, item }
}

impl<L: Lex, T: Lex> Lex for SkipThen<L, T> {
    fn lex<'i>(&self, input: &'i str) -> crate::LexResult<'i> {
        let (_, remaining) = self.lexer.lex(input).offset(input)?;
        let (output, remaining) = self.item.lex(remaining).offset(input)?;
        Ok((output, remaining))
    }
}

impl<L: Lex, T: Parse> Parse for SkipThen<L, T> {
    type Output = <T as Parse>::Output;

    fn parse<'i>(&self, input: &'i str) -> crate::ParseResult<'i, Self::Output> {
        let (_, remaining) = self.lexer.lex(input).offset(input)?;
        let (output, remaining) = self.item.parse(remaining).offset(input)?;

        Ok((output, remaining))
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::{char, int, token, InProgressError, Lex, Parse};

    #[test]
    fn lexer_then_skip_lexer() -> Result<(), InProgressError<'static>> {
        let test = then_skip(token("..."), token("abc"));

        assert_eq!(test.lex("abc...")?, ("abc", ""));

        let test = then_skip(char('.').many(1..=3), token("abc"));

        assert_eq!(
            test.lex("abc"),
            Err(InProgressError::no_match("").offset("abc"))
        );
        assert_eq!(test.lex("abc.")?, ("abc", ""));
        assert_eq!(test.lex("abc..")?, ("abc", ""));
        assert_eq!(test.lex("abc...")?, ("abc", ""));
        assert_eq!(test.lex("abc....")?, ("abc", "."));
        assert_eq!(test.lex("xyz.."), Err(InProgressError::no_match("xyz..")));
        assert_eq!(test.lex("..xyz"), Err(InProgressError::no_match("..xyz")));

        let test = then_skip(token("abc"), char('.').many(1..=3));

        assert_eq!(test.lex("abc"), Err(InProgressError::no_match("abc")));
        assert_eq!(test.lex(".abc")?, (".", ""));
        assert_eq!(test.lex("..abc")?, ("..", ""));
        assert_eq!(test.lex("...abc")?, ("...", ""));
        assert_eq!(
            test.lex("....abc"),
            Err(InProgressError::no_match(".abc").offset("....abc"))
        );
        assert_eq!(
            test.lex("...xyz"),
            Err(InProgressError::no_match("xyz").offset("...xyz"))
        );
        assert_eq!(test.lex("..abcd")?, ("..", "d"));

        Ok(())
    }

    #[test]
    fn parser_then_skip_lexer() -> Result<(), InProgressError<'static>> {
        let test = then_skip(token("..."), int::<u8>());

        assert_eq!(test.parse("123...")?, (123, ""));

        let test = then_skip(char('.').many(1..=3), int::<u8>());

        assert_eq!(
            test.parse("123"),
            Err(InProgressError::no_match("").offset("123"))
        );
        assert_eq!(test.parse("123.")?, (123, ""));
        assert_eq!(test.parse("123..")?, (123, ""));
        assert_eq!(test.parse("123...")?, (123, ""));
        assert_eq!(test.parse("123....")?, (123, "."));

        let test = then_skip(token("..."), int::<u8>().then_skip(char(',')).many(1..=3));

        assert_eq!(
            test.parse("123,"),
            Err(InProgressError::no_match("").offset("123,"))
        );
        assert_eq!(test.parse("123,...")?, (vec![123], ""));
        assert_eq!(test.parse("123,....")?, (vec![123], "."));
        assert_eq!(test.parse("123,123,...")?, (vec![123, 123], ""));
        assert_eq!(test.parse("123,123,....")?, (vec![123, 123], "."));
        assert_eq!(test.parse("123,123,123,...")?, (vec![123, 123, 123], ""));
        assert_eq!(test.parse("123,123,123,....")?, (vec![123, 123, 123], "."));
        assert_eq!(
            test.parse("123..."),
            Err(InProgressError::no_match("...").offset("123..."))
        );
        Ok(())
    }

    #[test]
    fn lexer_skip_then_lexer() -> Result<(), InProgressError<'static>> {
        let test = skip_then(token("..."), token("abc"));

        assert_eq!(test.lex("...abc")?, ("abc", ""));

        let test = skip_then(char('.').many(1..=3), token("abc"));

        assert_eq!(test.lex("abc"), Err(InProgressError::no_match("abc")));
        assert_eq!(test.lex(".abc")?, ("abc", ""));
        assert_eq!(test.lex("..abc")?, ("abc", ""));
        assert_eq!(test.lex("...abc")?, ("abc", ""));
        assert_eq!(
            test.lex("....abc"),
            Err(InProgressError::no_match(".abc").offset("....abc"))
        );
        assert_eq!(test.lex("xyz.."), Err(InProgressError::no_match("xyz..")));
        assert_eq!(
            test.lex("..xyz"),
            Err(InProgressError::no_match("xyz").offset("..xyz"))
        );

        let test = skip_then(token("abc"), char('.').many(1..=3));

        assert_eq!(
            test.lex("abc"),
            Err(InProgressError::no_match("").offset("abc"))
        );
        assert_eq!(test.lex("abc.")?, (".", ""));
        assert_eq!(test.lex("abc..")?, ("..", ""));
        assert_eq!(test.lex("abc...")?, ("...", ""));
        assert_eq!(test.lex("abc....")?, ("...", "."));
        assert_eq!(
            test.lex("abcd.."),
            Err(InProgressError::no_match("d..").offset("abcd.."))
        );

        Ok(())
    }

    #[test]
    fn parser_skip_then_lexer() -> Result<(), InProgressError<'static>> {
        let test = skip_then(token("..."), int::<u8>());

        assert_eq!(test.parse("...123")?, (123, ""));

        let test = skip_then(char('.').many(1..=3), int::<u8>());

        assert_eq!(test.parse("123"), Err(InProgressError::no_match("123")));
        assert_eq!(test.parse(".123")?, (123, ""));
        assert_eq!(test.parse("..123")?, (123, ""));
        assert_eq!(test.parse("...123")?, (123, ""));
        assert_eq!(
            test.parse("....123"),
            Err(InProgressError::no_match(".123").offset("....123"))
        );

        let test = skip_then(token("..."), char('>').skip_then(int::<u8>()).many(1..=3));

        assert_eq!(test.parse(">123"), Err(InProgressError::no_match(">123")));
        assert_eq!(test.parse("...>123")?, (vec![123], ""));
        assert_eq!(
            test.parse("....>123"),
            Err(InProgressError::no_match(".>123").offset("....>123"))
        );
        assert_eq!(test.parse("...>123>123")?, (vec![123, 123], ""));
        assert_eq!(
            test.parse("....>123>123"),
            Err(InProgressError::no_match(".>123>123").offset("....>123>123"))
        );
        assert_eq!(test.parse("...>123>123>123")?, (vec![123, 123, 123], ""));
        assert_eq!(
            test.parse("....>123>123>123"),
            Err(InProgressError::no_match(".>123>123>123").offset("....>123>123>123"))
        );
        assert_eq!(
            test.parse("...123"),
            Err(InProgressError::no_match("123").offset("...123"))
        );
        Ok(())
    }
}
