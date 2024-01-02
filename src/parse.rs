use std::ops::RangeBounds;

use crate::{
    combinator::{
        all, count, many, optional, or, pad,
        sequence::{All, LexMany},
        then, then_skip, Many, Optional, Or, Pad, Then, ThenSkip,
    },
    end, ws, End, Lex, WhiteSpace,
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
pub type ParseResult<'i, O> = Result<(O, &'i str), crate::Error<'i>>;

/// This trait is implemented by all Parsely parsers.
///
/// Its principle method is [`parse`] which takes an input `&str` and returns the matched part of the input, along with any remaining unmatched input.
///
/// We'll refer to types that implement [`Parse`] as parsers. See the [`parser`] module for a list of Parsley's built in parsers.
///
/// Most parsers you write will be composed of lexers that have their match mapped to your custom type. This is done using the [`map()`] method on a lexer
///
/// [`parse`]: Parse::parse
/// [`parser`]: crate::parser
/// [`lexer`]: crate::lexer
/// [`map()`]: crate::Lex::map
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
    fn many(self, range: impl RangeBounds<usize>) -> Many<Self, Vec<<Self as Parse>::Output>>
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
    fn count(self, n: usize) -> Many<Self, Vec<<Self as Parse>::Output>>
    where
        Self: Sized,
    {
        count(n, self)
    }

    /// Creates a new parser that will attempt to parse with this parser multiple times until end of input.
    ///
    /// See [`crate::combinator::all()`] and the [`sequence module`](crate::combinator::sequence) for more details.
    fn all(self, min: usize) -> All<Self, Vec<<Self as Parse>::Output>>
    where
        Self: Sized,
    {
        all(min, self)
    }

    /// Creates a new parser from this one that will match 0 or 1 times, making it optional.
    ///
    /// The output is wrapped in an [`Option`]: if this parser doesn't match it outputs a `None`.
    ///
    /// This means `.optional()` is **not** equivalent to `.many(0..=1)` which outputs into a [`Vec`].
    ///
    /// # Examples
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
    /// ```
    /// use parsely::{token, Lex, Parse};
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
    /// use parsely::{Lex, Parse, ParseResult};
    ///
    /// # #[derive(Debug, PartialEq)]
    /// # enum FooBar {
    /// #     Foo,
    /// #     Bar,
    /// # }
    /// fn parse_foo_bar<'i>(input: &'i str) -> ParseResult<'i, FooBar> {
    ///     "foo".map(|_| FooBar::Foo)
    ///         .or("floobydoobyfooo".map(|_| FooBar::Foo))
    ///         .or("babababarrr".map(|_| FooBar::Bar))
    ///         .or("bar".map(|_| FooBar::Bar)).parse(input)
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
    ///     "foo".or("floobydoobyfooo").map(|_| FooBar::Foo).or(
    ///         "bar".or("babababarrr").map(|_| FooBar::Bar)
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
    /// ```
    /// use parsely::{Lex, Parse};
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
    /// let homer = "Homer".map(|_| Homer);
    /// let marge = "Marge".map(|_| Marge);
    /// let bart = "Bart".map(|_| Bart);
    /// let lisa = "Lisa".map(|_| Lisa);
    /// let maggie = "Maggie".map(|_| Maggie);
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
    fn then<P>(self, parser: P) -> Then<Self, P>
    where
        Self: Sized,
    {
        then(self, parser)
    }

    /// Creates a parser that runs a lexer on the remaining input after running this parser.
    ///
    /// The output of the lexer is ignored, or "skipped".
    ///
    /// See also [`Lex::skip_then`] which runs and ignores a lexer *before* running a parser.
    ///
    /// This is useful when there is filler input that isn't relevant to what is being parsed that you need to match but don't want to map.
    ///
    /// # Examples
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

    /// This "finalizes" the parser, which means it expects there to be no remaining input.
    ///
    /// If any input remains after parsing, then the whole parse fails.
    ///
    /// This is a convenience method alternative to using `.then_skip(end())` which saves importing [`end()`]
    fn then_end(self) -> ThenSkip<End, Self>
    where
        Self: Sized,
    {
        self.then_skip(end())
    }

    /// Map the output of this parser to some other type.
    fn map<F, O>(self, f: F) -> Mapped<Self, F>
    where
        F: Fn(<Self as Parse>::Output) -> O,
        Self: Sized,
    {
        Mapped { f, parser: self }
    }

    /// Swaps around the tuple output by the [`then()`] parser.
    ///
    /// * `a.then(b)` outputs `(a, b)`
    /// * `a.then(b).swap()` outputs `(b, a)`.
    ///
    /// This is simply a convenience method for doing `.map(|(a, b)| (b, a))`.
    ///
    /// # Examples
    ///
    /// ```
    /// use parsely::{int, switch, Parse};
    ///
    /// # use std::collections::HashMap;
    ///
    /// let int_then_color = int::<u8>().pad().then(switch([
    ///     ("red", Color::Red),
    ///     ("green", Color::Green),
    ///     ("blue", Color::Blue),
    /// ]));
    ///
    /// let (output, remaining) = int_then_color.parse("1 red")?;
    /// assert_eq!(output, (1, Color::Red));
    ///
    /// let (output, remaining) = int_then_color.swap().many(..).delimiter(',').parse("1 red, 2 blue, 3 green")?;
    /// assert_eq!(&output[2], &(Color::Green, 3));
    ///
    /// // swap() made collecting into a HashMap convenient :)
    /// let hashmap = output.into_iter().collect::<HashMap<_, _>>();
    /// assert_eq!(hashmap.get(&Color::Green), Some(&3));
    ///
    /// // note that Clone is required to use `switch`, and Eq + Hash is required to collect into a HashMap.
    /// #[derive(Clone, Copy, Debug, PartialEq, Hash, Eq)]
    /// pub enum Color {
    ///     Red,
    ///     Green,
    ///     Blue,
    /// }
    /// # Ok::<(), parsely::Error>(())
    /// ```
    fn swap(self) -> Swapped<Self>
    where
        <Self as Parse>::Output: Swap,
        Self: Sized,
    {
        Mapped::new(<<Self as Parse>::Output as Swap>::swap, self)
    }

    /// Pad this parser with zero or more whitespace lexers so that leading and/or trailing whitespace in the input doesn't interfere with parsing
    ///
    /// WARNING: `.pad()` leads to suprising bugs when included inside `.then()`. See the [**Combining pad with then** example](pad).
    ///
    /// This is an opinionated default usage of the pad combinator for convenience.
    ///
    /// The pad combinator will accept arbitrary lexers for the left and right side. See it's documentation for more details.
    ///
    /// # Examples
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

    /// Pad this parser with the given left and right lexers.
    ///
    /// See also [`pad()`](Lex::pad()) which pads with zero or more whitepsace characters by default.
    ///
    /// # Examples
    ///
    /// ```
    /// use parsely::{int, Parse};
    ///
    /// let parser = int::<u8>().pad_with('[', ']');
    ///
    /// assert_eq!(parser.parse("[123]")?, (123, ""));
    /// # Ok::<(), parsely::Error>(())
    /// ```
    fn pad_with<L: Lex, R: Lex>(self, left: L, right: R) -> Pad<L, R, Self>
    where
        Self: Sized,
    {
        pad(left, right, self)
    }
}

/// Maps the output of a parser to a different output
pub struct Mapped<P, F> {
    f: F,
    parser: P,
}

impl<P, F> Mapped<P, F> {
    pub fn new(f: F, parser: P) -> Self {
        Mapped { f, parser }
    }
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
/// use parsely::{hex, Lex, Parse, ParseResult};
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
/// let (output, remaining) = '#'.skip_then(hex_rgb).parse("#AABBCC")?;
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

/// swapped type
pub type Swapped<T> =
    Mapped<T, fn(<T as Parse>::Output) -> <<T as Parse>::Output as Swap>::Swapped>;

/// Used to conveniently swap a tuple t'other way round.
///
/// Currently this only really exists to provide a named function in the definition of [`Then::swap()`](Then::swap)
pub trait Swap {
    type Swapped;

    fn swap(self) -> Self::Swapped;
}

impl<A, B> Swap for (A, B) {
    type Swapped = (B, A);

    fn swap(self) -> Self::Swapped {
        let (a, b) = self;
        (b, a)
    }
}
