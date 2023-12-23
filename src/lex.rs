use std::ops::RangeBounds;

use crate::{
    combinator::{
        count, many, map, optional, or, pad, sequence::LexMany, skip_then, then, then_skip,
        try_map, Many, Map, Optional, Or, Pad, SkipThen, Then, ThenSkip, TryMap,
    },
    ws, Parse, WhiteSpace,
};

/// The type returned by a lex: the order of the tuple is `(matched, remaining)`
///
/// * First the part of the input successfully matched
/// * Then the remaining part of the input that was not matched.
///
/// The order reads left to right as the lexer reads the input, and matches the return order of [`str::split_at`].
///
/// Often the lifetime parameter can be elided:
/// ```rust
/// # use parsely::{LexResult};
/// fn my_lexer(input: &str) -> LexResult<'_> {
///     // ...
///     # Ok((input, ""))
/// }
/// ```
pub type LexResult<'i> = Result<(&'i str, &'i str), crate::Error<'i>>;

/// This trait is implemented by all Parsely lexers.
///
/// Its principle method is [`lex`](Lex::lex) which takes an input `&str` and returns the matched part of the input, along with any remaining input.
///
/// By repeating this process and mapping the matched parts of the input to your types, you will create a parser.
///
/// This is useful to break apart large complex input into smaller pieces which can be processed by parsers into other types.
///
/// Most Parsely parser combinators will be built up from primitives that implement Lex such as [`char()`], [`token()`] and [`digit()`].
///
/// We'll refer to types that implement [`Lex`] as Lexers.
///
/// Lexers can be combined using combinators. That's what the majority of the methods in this trait provide: convenient ways to combine different lexers and parsers together.
///
/// The [`combinator`] module defines the concrete types that these methods return.
///
/// [`lex`]: Lex::lex
/// [`char()`]: crate::char
/// [`token()`]: crate::token
/// [`digit()`]: crate::digit
/// [`combinator`]: crate::combinator
pub trait Lex {
    /// Match part or all of an input str, breaking it down into smaller pieces to make parsing easier.
    fn lex<'i>(&self, input: &'i str) -> LexResult<'i>;

    /// Creates a new lexer that will attempt to lex with this lexer multiple times.
    ///
    /// See [`crate::combinator::many()`] for more details.
    fn many(self, range: impl RangeBounds<usize>) -> Many<Self, Vec<()>>
    where
        Self: Sized,
    {
        many(range, self)
    }

    /// Creates a new lexer that will attempt to lex with this lexer exactly n times.
    ///
    /// This is equivalent to `.many(n..=n)`.
    ///
    /// See [`crate::combinator::Many`] for more details.
    fn count(self, n: usize) -> Many<Self, Vec<()>>
    where
        Self: Sized,
    {
        count(n, self)
    }

    /// Creates a new lexer from this one that will match 0 or 1 times, making it optional.
    ///
    /// This is equivalent to `.many(0..=1)`. Using `.optional()` is preferred for legibility.
    fn optional(self) -> Optional<Self>
    where
        Self: Sized,
    {
        optional(self)
    }

    /// Creates a new lexer that will attempt to lex with this lexer, and if it fails, attempt to lex with the given lexer.
    ///
    /// This can be used to build a chain of possible ways to lex the same input.
    ///
    /// At most, one of the lexers will consume input.
    ///
    /// # Examples
    ///
    /// Basic usage:
    ///
    /// ```
    /// use parsely::{char, token, Lex};
    ///
    /// let for_or_bar = token("foo").or(token("bar"));
    ///
    /// let (output, remaining) = for_or_bar.lex("foobarbaz")?;
    ///
    /// assert_eq!(output, "foo");
    /// assert_eq!(remaining, "barbaz");
    ///
    /// let (output, remaining) = for_or_bar.lex("barbaz")?;
    ///
    /// assert_eq!(output, "bar");
    /// assert_eq!(remaining, "baz");
    ///
    /// // `or` can be chained multiple times:
    ///
    /// let whitespace = char(' ')
    ///     .or(char('\t'))
    ///     .or(char('\n'))
    ///     .or(char('\r'));
    ///
    /// # Ok::<(), parsely::Error>(())
    /// ```
    ///
    /// Note that there is a whitespace lexer available, see [`crate::lexer::ws`]
    fn or<L: Lex>(self, lexer: L) -> Or<Self, L>
    where
        Self: Sized,
    {
        or(self, lexer)
    }

    /// Creates a new lexer that applies two lexers in sequence.
    ///
    /// First this lexer is run, and then if successful, the remaining input will be fed to the given lexer.
    ///
    /// This lexer short circuits such that if the first lexer does not match, the second one is not attempted.
    ///
    /// Both lexers are required to match for any input to be consumed.
    ///
    /// # Examples
    ///
    /// Basic usage:
    ///
    /// ```
    /// use parsely::{char, hex, Lex};
    ///
    /// let hex_color = char('#').then(hex().many(1..));
    ///
    /// let (output, remaining) = hex_color.lex("#C0FFEE")?;
    ///
    /// assert_eq!(output, "#C0FFEE");
    /// assert_eq!(remaining, "");
    ///
    /// let result = hex_color.lex("#TEATEA");
    ///
    /// assert!(result.is_err());
    ///
    /// # Ok::<(), parsely::Error>(())
    /// ```
    fn then<L: Lex>(self, lexer: L) -> Then<Self, L>
    where
        Self: Sized,
    {
        then(self, lexer)
    }

    /// Run this lexer, and then another item.
    ///
    /// The output of the item is ignored, or "skipped".
    ///
    /// See also [`Lex::skip_then`] and [`Parse::then_skip`].
    ///
    /// This is useful when there is input you need to match but don't want to keep as part of the match.
    ///
    /// For a full side by side comparison of all the `skip_then()` and `then_skip()` methods see the [`skip module`](crate::combinator::skip) documentation.
    ///
    /// # Examples
    ///
    /// Basic usage:
    ///
    /// ```
    /// use parsely::{digit, token, Lex};
    ///
    /// let (output, remaining) = digit().many(..).then_skip(token("<<<")).lex("123<<<")?;
    /// assert_eq!(output, "123");
    /// assert_eq!(remaining, "");
    ///
    /// # Ok::<(), parsely::Error>(())
    /// ```
    fn then_skip<L: Lex>(self, lexer: L) -> ThenSkip<L, Self>
    where
        Self: Sized,
    {
        then_skip(lexer, self)
    }

    /// Creates a parser that runs parses the remaining input after running this lexer.
    ///
    /// The output of this lexer is ignored, or "skipped".
    ///
    /// See also [`Parse::then_skip`] which runs and ignores a lexer *after* running a parser.
    ///
    /// This is useful when there is filler input that isn't relevant to what is being parsed that you need to match but don't want to map.
    ///
    /// # Examples
    ///
    /// Basic usage:
    ///
    /// ```
    /// use parsely::{int, token, Lex, Parse, ParseResult};
    ///
    /// fn parser(input: &str) -> ParseResult<'_, u8> {
    ///     token(">>>").skip_then(int::<u8>()).parse(input)
    /// }
    ///
    /// let (output, remaining) = parser(">>>123")?;
    /// assert_eq!(output, 123);
    /// assert_eq!(remaining, "");
    ///
    /// # Ok::<(), parsely::Error>(())
    /// ```
    fn skip_then<P: Parse>(self, parser: P) -> SkipThen<Self, P>
    where
        Self: Sized,
    {
        skip_then(self, parser)
    }

    /// Creates a parser by mapping the matched part of this lexer to an output type.
    ///
    /// This is best for mapping specific known tokens. If the conversion might fail you must use [`Lex::try_map()`] instead.
    ///
    /// # Examples
    ///
    /// Basic usage:
    ///
    /// ```
    /// use std::net::Ipv4Addr;
    ///
    /// use parsely::{token, Lex, Parse};
    ///
    /// let parser = token("localhost").map(|_| Ipv4Addr::new(127, 0, 0, 1));
    ///
    /// let (output, remaining) = parser.parse("localhost")?;
    /// assert_eq!(output, Ipv4Addr::LOCALHOST);
    ///
    /// # Ok::<(), parsely::Error>(())
    /// ```
    fn map<F, O>(self, f: F) -> Map<Self, F>
    where
        Self: Sized,
        F: Fn(&str) -> O,
    {
        map(self, f)
    }

    /// Creates a parser by mapping the matched part of this lexer to an output type.
    ///
    /// Unlike [`map()`], this returns a `Result<T, parsely::Error>` in case of failed conversions.
    ///
    /// This is needed to map matches using [`std::str::FromStr`].
    ///
    /// # Examples
    ///
    /// Basic usage:
    ///
    /// ```
    /// use std::{net::Ipv4Addr, str::FromStr};
    ///
    /// use parsely::{char, digit, Lex, Parse};
    ///
    /// fn bad_ip_parser() -> impl Parse<Output=Ipv4Addr> {
    ///     digit().many(1..=3).count(4).delimiter(char('.')).try_map(FromStr::from_str)
    /// }
    ///
    /// let (output, remaining) = bad_ip_parser().parse("127.0.0.1")?;
    /// assert_eq!(output, Ipv4Addr::LOCALHOST);
    ///
    /// # Ok::<(), parsely::Error>(())
    /// ```
    fn try_map<F, O, E>(self, f: F) -> TryMap<Self, F>
    where
        Self: Sized,
        F: Fn(&str) -> Result<O, E>,
    {
        try_map(self, f)
    }

    /// Pad this lexer with zero or more whitespace lexers so that leading and/or trailing whitespace in the input is ignored.
    ///
    /// This is an opionated default usage of the pad combinator for convenience.
    ///
    /// The pad combinator will accept arbitrary lexers for the left and right side. See it's documentation for more details.
    ///
    /// # Examples
    ///
    /// Basic usage:
    ///
    /// ```
    /// use parsely::{int, Parse};
    ///
    /// assert_eq!(
    ///     int::<u32>().pad().parse("   123\n")?,
    ///     (123, "")
    /// );
    ///
    /// assert_eq!(
    ///     int::<u32>().pad().many(1..).parse("   123\n\t456\t789\r\n    10")?,
    ///     (vec![123, 456, 789, 10], "")
    /// );
    /// # Ok::<(), parsely::Error>(())
    /// ```
    fn pad(self) -> Pad<LexMany<WhiteSpace>, LexMany<WhiteSpace>, Self>
    where
        Self: Sized,
    {
        pad(ws().many(0..), ws().many(0..), self)
    }

    /// Pad this lexer with the given left and right lexers.
    ///
    /// See also [`pad()`](Parse::pad()) which pads with zero or more whitepsace characters by default.
    ///
    /// # Examples
    ///
    /// Basic usage:
    ///
    /// ```
    /// use parsely::{char, digit, Lex};
    ///
    /// let lexer = digit().pad_with(char('['), char(']'));
    ///
    /// assert_eq!(lexer.lex("[1]")?, ("1", ""));
    /// # Ok::<(), parsely::Error>(())
    /// ```
    fn pad_with<L: Lex, R: Lex>(self, left: L, right: R) -> Pad<L, R, Self>
    where
        Self: Sized,
    {
        pad(left, right, self)
    }
}
impl<F> Lex for F
where
    F: Fn(&str) -> LexResult<'_>,
{
    /// Functions that take &str and return `Result<(&str, &str), parsely::Error>` are Lexers.
    ///
    /// The matched part of the input str is returned on the left hand side.
    ///
    /// The remaining part of the input str is returned on the right hand side.
    ///
    /// This is the same order that [`str::split_at()`] returns.
    ///
    /// > **Note**: To get proper error spans from your functions you must:
    /// > * call `.offset(input)` on any errors returned by built in lexers
    /// > * use `parsely::Error::no_match(input)` to create an Error
    /// >
    /// > where `input` is the argument to your function that is the input to be lexed
    ///
    /// # Examples
    ///
    /// This example uses the alias [`LexResult`] for the return type, and [elides] the lifetime parameter.
    ///
    /// [elides]: https://doc.rust-lang.org/nomicon/lifetime-elision.html
    ///
    /// The equivalent expanded signature is `fn my_lexer(input: &str) -> Result<(&str, &str), parsely::Error<'_>>`
    /// ```
    /// use parsely::{digit, Lex, LexResult};
    ///
    /// fn my_lexer(input: &str) -> LexResult<'_> {
    ///     let boundary = input.find("abc").ok_or(parsely::Error::no_match(input))?;
    ///     let (output, remaining) = input.split_at(boundary + 3);
    ///
    ///     Ok((output, remaining))
    /// }
    ///
    /// // this lexer function matches up to and including the token "abc"
    /// let (output, remaining) = my_lexer("...abc")?;
    /// assert_eq!(output, "...abc");
    ///
    /// // because it implements Lex, we can use it to build a more complex lexer chain
    /// let (output, remaining) = my_lexer.then(digit().many(1..=3)).count(3).lex("...abc123.abc123..abc123...")?;
    /// assert_eq!(output, "...abc123.abc123..abc123");
    /// assert_eq!(remaining, "...");
    ///
    /// # Ok::<(), parsely::Error>(())
    /// ```
    ///
    /// Note the lack of `()` after `my_lexer` when it is used as part of a chain.
    /// The function itself is being used, it won't be executed until you call `.lex()`
    ///
    fn lex<'i>(&self, input: &'i str) -> LexResult<'i> {
        self(input)
    }
}

impl Lex for char {
    /// [`Lex`] is implemented for [`char`]
    ///
    /// It works the same way as the [`char()`](crate::char) lexer.
    fn lex<'i>(&self, input: &'i str) -> LexResult<'i> {
        let mut chars = input.char_indices();

        match chars.next() {
            Some((_, c)) if c == *self => {
                let boundary = match chars.next() {
                    Some((n, _)) => n,
                    None => input.len(),
                };

                Ok(input.split_at(boundary))
            }
            _ => Err(crate::Error::no_match(input)),
        }
    }
}

impl<'a> Lex for &'a str {
    /// [`Lex`] is implemented for [`str`]
    ///
    /// It works the same way as the [`token()`](crate::token) lexer.
    fn lex<'i>(&self, input: &'i str) -> LexResult<'i> {
        if input.starts_with(self) {
            Ok(input.split_at(self.len()))
        } else {
            Err(crate::Error::no_match(input))
        }
    }
}
