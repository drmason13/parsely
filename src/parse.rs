use std::ops::RangeBounds;

use crate::{
    combinator::{count, many, optional, or, then, then_skip, Many, Optional, Or, Then, ThenSkip},
    Lex,
};

/// The type returned by a parse. The order of the tuple is `(output, remaining)`
///
/// * First the output of the parser
/// * Then the remaining part of the input.
///
/// The order reads left to right as the parser reads the input, and matches the return order of [`str::split_at`].
///
/// Often the lifetime parameter can be elided:
/// ```rust
/// # use parsely::{ParseResult};
/// # struct Foo;
/// fn my_parser(input: &str) -> ParseResult<'_, Foo> {
///     // ...
///     # Ok((Foo, ""))
/// }
/// ```
pub type ParseResult<'i, O> = Result<(O, &'i str), crate::Error>;

/// This trait is implemented by all Parsely parsers.
///
/// Its principle method is [`parse`](Parse::parse) which takes an input `&str` and returns the matched part of the input, along with any remaining unmatched input.
///
/// This is useful to break apart large complex input into smaller pieces which can be processed by parsers into other types.
pub trait Parse {
    /// The output type produced by a successful parse.
    type Output;

    /// Parse a string input into the output type (`Self::Output`) and return any remaining input.
    ///
    /// This method returns a tuple `(output, remaining)` of `&str`.
    ///
    /// * First is the output of the parser.
    /// * Second is the remaining part of the input that was not matched.
    ///
    /// This order reads left to right as the parser reads the input, and matches the return order of [`str::split_at`].
    fn parse<'i>(&self, input: &'i str) -> ParseResult<'i, Self::Output>;

    /// Creates a new parser that will attempt to parse with this parser multiple times.
    ///
    /// See [`crate::combinator::many()`] and the [`sequence module`](crate::combinator::sequence) for more details.
    fn many(self, range: impl RangeBounds<usize>) -> Many<Self>
    where
        Self: Sized,
    {
        many(range, self)
    }

    /// Creates a new parser that will attempt to parse with this parser exactly n times.
    ///
    /// This is equivalent to `.many(n..=n)`.
    ///
    /// See [`crate::combinator::Many`] for more details.
    fn count(self, n: usize) -> Many<Self>
    where
        Self: Sized,
    {
        count(n, self)
    }

    /// Creates a new parser from this one that will match 0 or 1 times, making it optional.
    ///
    /// The output is wrapped in an [`Option`]: if this parser doesn't match it outputs a `None`.
    ///
    /// This means `.optional()` is **not** equivalent to `.many(0..=1)` which outputs into a [`Vec`].
    ///
    /// # Examples
    ///
    /// Basic usage:
    ///
    /// ```
    /// use parsely::{int, token, Lex, Parse};
    ///
    /// let parser = int::<u32>().optional();
    ///
    /// let (output, remaining) = parser.clone().then(token("abc").map(|_| 7)).parse("123abc")?;
    /// assert_eq!(output, (Some(123), 7));
    ///
    /// let (output, remaining) = parser.parse("abc")?;
    /// assert_eq!(output, None);
    ///
    /// # Ok::<(), parsely::Error>(())
    /// ```
    fn optional(self) -> Optional<Self>
    where
        Self: Sized,
    {
        optional(self)
    }

    /// Creates a new parser that will attempt to parse with this parser, and if it fails, attempt to parse with the given parser.
    ///
    /// This can be used to build a chain of possible ways to parse the same input.
    ///
    /// At most, one of the parsers will consume input.
    ///
    /// # Examples
    ///
    /// Basic usage:
    ///
    /// ```
    /// use parsely::{char, token, Lex, Parse};
    ///
    /// # #[derive(Debug, PartialEq)]
    /// enum FooBar {
    ///     Foo,
    ///     Bar,
    /// }
    ///
    /// fn parse_foo_bar(input: &str) -> Result<(FooBar, &str), parsely::Error> {
    ///     token("foo").map(|_| FooBar::Foo)
    ///         .or(token("bar").map(|_| FooBar::Bar)).parse(input)
    /// }
    ///
    /// let (output, remaining) = parse_foo_bar("foobarbaz")?;
    ///
    /// assert_eq!(output, FooBar::Foo);
    /// assert_eq!(remaining, "barbaz");
    ///
    /// let (output, remaining) = parse_foo_bar("barbaz")?;
    ///
    /// assert_eq!(output, FooBar::Bar);
    /// assert_eq!(remaining, "baz");
    /// # Ok::<(), parsely::Error>(())
    /// ```
    ///
    /// Chained and nested or:
    /// ```
    /// use parsely::{char, token, Lex, Parse, ParseResult};
    ///
    /// # #[derive(Debug, PartialEq)]
    /// # enum FooBar {
    /// #     Foo,
    /// #     Bar,
    /// # }
    /// fn parse_foo_bar<'i>(input: &'i str) -> ParseResult<'i, FooBar> {
    ///     token("foo").map(|_| FooBar::Foo)
    ///         .or(token("floobydoobyfooo").map(|_| FooBar::Foo))
    ///         .or(token("babababarrr").map(|_| FooBar::Bar))
    ///         .or(token("bar").map(|_| FooBar::Bar)).parse(input)
    /// }
    ///
    /// let (output, remaining) = parse_foo_bar("babababarrr is a Bar")?;
    ///
    /// assert_eq!(output, FooBar::Bar);
    /// assert_eq!(remaining, " is a Bar");
    ///
    /// // or can be nested, so parse_foo_bar can be written as:
    ///
    /// fn parse_foo_bar_nested<'i>(input: &'i str) -> ParseResult<'i, FooBar> {
    ///     token("foo").or(token("floobydoobyfooo")).map(|_| FooBar::Foo).or(
    ///         token("bar").or(token("babababarrr")).map(|_| FooBar::Bar)
    ///     ).parse(input)
    /// }
    ///
    /// let (output, remaining) = parse_foo_bar_nested("floobydoobyfooo is a Foo too")?;
    ///
    /// assert_eq!(output, FooBar::Foo);
    /// assert_eq!(remaining, " is a Foo too");
    ///
    /// # Ok::<(), parsely::Error>(())
    /// ```
    fn or<P: Parse>(self, parser: P) -> Or<Self, P>
    where
        Self: Sized,
    {
        or(self, parser)
    }

    /// Creates a new parser that applies two parsers in sequence.
    ///
    /// First this parser is run, and then if successful, the remaining input will be fed to the given parser.
    ///
    /// This parser short circuits such that if the first parser does not match, the second one is not attempted.
    ///
    /// Both parsers are required to match for any input to be consumed.
    ///
    /// See also [`Lex::then`] which applies two lexers in sequence.
    ///
    /// See [`Parse::then_skip`] and [`Lex::skip_then`] to mix lexers and parsers in sequence.
    ///
    /// # Examples
    ///
    /// Basic usage:
    ///
    /// ```
    /// use parsely::{int, token, Lex, Parse, ParseResult};
    ///
    /// # #[derive(Debug, PartialEq)]
    /// enum Simpson {
    ///     Homer,
    ///     Marge,
    ///     Bart,
    ///     Lisa,
    ///     Maggie,
    /// }
    ///
    /// use Simpson::*;
    ///
    /// let homer = token("Homer").map(|_| Homer);
    /// let marge = token("Marge").map(|_| Marge);
    /// let bart = token("Bart").map(|_| Bart);
    /// let lisa = token("Lisa").map(|_| Lisa);
    /// let maggie = token("Maggie").map(|_| Maggie);
    ///
    /// let parser = homer.then(marge).then(lisa).then(maggie).then(bart);
    ///
    /// let (output, remaining) = parser.parse("HomerMargeLisaMaggieBartMilhouse")?;
    ///
    /// assert_eq!(output, ((((Homer, Marge), Lisa), Maggie), Bart));
    /// assert_eq!(remaining, "Milhouse");
    ///
    /// # Ok::<(), parsely::Error>(())
    /// ```
    fn then<P: Parse>(self, parser: P) -> Then<Self, P>
    where
        Self: Sized,
    {
        then(self, parser)
    }

    /// Creates a parser that runs a lexer on the remaining input after running this parser.
    ///
    /// The output of the lexer is ignored, or "skipped".
    ///
    /// See also [`Lex::skip_then`] which runs and ignores a lexer *before* running this parser.
    ///
    /// This is useful when there is filler input that isn't relevant to what is being parsed that you need to match but don't want to map.
    ///
    /// # Examples
    ///
    /// Basic usage:
    ///
    /// ```
    /// use parsely::{int, token, Parse, ParseResult};
    ///
    /// #[derive(Debug, PartialEq)]
    /// struct Foo;
    ///
    /// fn parser(input: &str) -> ParseResult<'_, u8> {
    ///     int::<u8>().then_skip(token("<<<")).parse(input)
    /// }
    ///
    /// let (output, remaining) = parser("123<<<")?;
    /// assert_eq!(output, 123);
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

    /// Map the output of this parser to some other type.
    fn map<F, O>(self, f: F) -> Mapped<Self, F>
    where
        F: Fn(<Self as Parse>::Output) -> O,
        Self: Sized,
    {
        Mapped { f, parser: self }
    }
}

/// Maps the output of a parser to a different output
pub struct Mapped<P, F> {
    f: F,
    parser: P,
}

impl<P, F, O1, O2> Parse for Mapped<P, F>
where
    P: Parse<Output = O1>,
    F: Fn(O1) -> O2,
{
    type Output = O2;

    fn parse<'i>(&self, input: &'i str) -> ParseResult<'i, Self::Output> {
        let (output, remaining) = self.parser.parse(input)?;
        let mapped = (self.f)(output);
        Ok((mapped, remaining))
    }
}

/// Functions that take &str and return `Result<(O, &str), parsely::Error>` impl Parse and can be used with Parsely combinators.
///
/// The output of the parser is returned on the left hand side.
///
/// The remaining part of the input str is returned on the right hand side.
///
/// This means it is easy to create your own parser without implementing `Parse`.
///
/// # Examples
///
/// ```
/// use parsely::{char, digit, hex, Lex, Parse, ParseResult};
///
/// # #[derive(PartialEq, Eq, Debug)]
/// # struct Rgb(u8, u8, u8);
///
/// // Sometimes its easiest to just return some type that implements Parse outputting u8
/// fn hex_byte() -> impl Parse<Output=u8> {
///     hex().many(1..=2).try_map(|s| u8::from_str_radix(s, 16))
/// }
///
/// // Here we have a fn that *is* a parser, sometimes you might prefer to define your own parsers this way
/// fn hex_rgb(input: &str) -> ParseResult<'_, Rgb> {
///    let (((r, g), b), remaining) = hex_byte().then(hex_byte()).then(hex_byte()).parse(input)?;
///    Ok((Rgb(r, g, b), remaining))
/// };
///
/// // because hex_rgb implements Parse, we can use it to build a more complex parser chain
/// let (output, remaining) = char('#').skip_then(hex_rgb).parse("#AABBCC")?;
/// assert_eq!(output, Rgb(170, 187, 204));
///
/// # Ok::<(), parsely::Error>(())
/// ```
///
/// The type alias has a lifetime parameter but it can usually be elided: ```_``.
/// It's the lifetime `'i` of the input string:  `&'i str`
impl<F, O> Parse for F
where
    F: Fn(&str) -> Result<(O, &str), crate::Error>,
{
    type Output = O;

    fn parse<'i>(&self, input: &'i str) -> ParseResult<'i, O> {
        self(input)
    }
}
