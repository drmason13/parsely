use std::{fmt, ops::RangeBounds};

use crate::{
    combinator::{count, many, or, then, Many, Or, Then},
    LexError,
};

#[non_exhaustive]
#[derive(Debug, PartialEq)]
pub enum ParseError {
    NoMatch,
    FailedConversion,
}

impl std::error::Error for ParseError {}
impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ParseError::NoMatch => write!(f, "No Match"),
            ParseError::FailedConversion => write!(f, "Failed to convert matched input"),
        }
    }
}

impl From<LexError> for ParseError {
    fn from(value: LexError) -> Self {
        match value {
            LexError::NoMatch => ParseError::NoMatch,
        }
    }
}

pub type ParseResult<'i, O> = Result<(O, &'i str), ParseError>;

/// This trait is implemented by all Parsely parsers.
///
/// Its principle method is [`parse`](Parse::parse) which takes an input `&str` and returns the matched part of the input, along with any remaining unmatched input.
///
/// This is useful to break apart large compparse input into smaller pieces which can be processed by parsers into other types.
pub trait Parse: Sized {
    /// The output type produced by a successful parse.
    type Output;

    /// The  method returns a tuple `(matched, remaining)` of `&str`.
    ///
    /// First the part of the input successfully matched and then the remaining part of the input that was not matched.
    ///
    /// The order reads left to right as the parser reads the input, and matches the return order of [`str::split_at`].
    fn parse<'i>(&mut self, input: &'i str) -> ParseResult<'i, Self::Output>;

    /// Creates a new parser that will attempt to parse with this parser multiple times.
    ///
    /// See [`combinators::Many`] for more details.
    fn many(self, range: impl RangeBounds<usize>) -> Many<Self>
    where
        Self: Sized,
    {
        many(range, self)
    }

    /// Creates a new parser that will attempt to parse with this parser exactly n times.
    ///
    /// See [`combinators::Many`] for more details.
    fn count(self, n: usize) -> Many<Self>
    where
        Self: Sized,
    {
        count(n, self)
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
    /// use parsely::{char, token, Lex, Parse, ParseError};
    ///
    /// # #[derive(Debug, PartialEq)]
    /// enum FooBar {
    ///     Foo,
    ///     Bar,
    /// }
    ///
    /// fn parse_foo_bar(input: &str) -> Result<(FooBar, &str), ParseError> {
    ///     token("foo").map(|_| Ok::<FooBar, ParseError>(FooBar::Foo))
    ///         .or::<_, FooBar>(token("bar").map(|_| Ok::<FooBar, ParseError>(FooBar::Bar))).parse(input)
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
    /// # Ok::<(), ParseError>(())
    /// ```
    ///
    /// Chained and nested or:
    /// ```
    /// use parsely::{char, token, Lex, Parse, ParseError, ParseResult};
    ///
    /// # #[derive(Debug, PartialEq)]
    /// # enum FooBar {
    /// #     Foo,
    /// #     Bar,
    /// # }
    /// fn parse_foo_bar<'i>(input: &'i str) -> ParseResult<'i, FooBar> {
    ///     token("foo").map(|_| Ok::<FooBar, ParseError>(FooBar::Foo))
    ///         .or::<_, FooBar>(token("floobydoobyfooo").map(|_| Ok::<FooBar, ParseError>(FooBar::Foo)))
    ///         .or::<_, FooBar>(token("babababarrr").map(|_| Ok::<FooBar, ParseError>(FooBar::Bar)))
    ///         .or::<_, FooBar>(token("bar").map(|_| Ok::<FooBar, ParseError>(FooBar::Bar))).parse(input)
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
    ///     token("foo").or(token("floobydoobyfooo")).map(|_| Ok::<FooBar, ParseError>(FooBar::Foo))
    ///         .or::<_, FooBar>(token("bar").or(token("babababarrr")).map(|_| Ok::<FooBar, ParseError>(FooBar::Bar))).parse(input)
    /// }
    ///
    /// let (output, remaining) = parse_foo_bar_nested("floobydoobyfooo is a Foo too")?;
    ///
    /// assert_eq!(output, FooBar::Foo);
    /// assert_eq!(remaining, " is a Foo too");
    ///
    /// # Ok::<(), ParseError>(())
    /// ```
    ///
    /// Note that there is a whitespace parser available, see [`parsers::ws`]
    fn or<P: Parse, O>(self, parser: P) -> Or<Self, P>
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
    /// # Examples
    ///
    /// Basic usage:
    ///
    /// ```
    /// use parsely::{char, hex, Lex, Parse, ParseError, ParseResult};
    ///
    /// # #[derive(Debug, PartialEq)]
    /// pub struct Rgb(u8, u8, u8);
    ///
    /// fn hex_rgb<'i>(input: &'i str) -> ParseResult<'i, Rgb> {
    ///     let (_, remaining) = char('#').lex(input)?;
    ///     let hex_color = hex().count(2).map(|s| u8::from_str_radix(s, 16));
    ///
    ///     let (output, remaining) = hex_color.count(3).parse(remaining)?;
    ///     let mut colors = output.iter().copied();
    ///     let r = colors.next().ok_or(ParseError::NoMatch)?;
    ///     let g = colors.next().ok_or(ParseError::NoMatch)?;
    ///     let b = colors.next().ok_or(ParseError::NoMatch)?;
    ///
    ///     Ok((Rgb(r, g, b), remaining))
    /// };
    ///
    /// //let (output, remaining) = hex_rgb.parse("#C0FFEE")?;
    ///
    /// //assert_eq!(output, Rgb(192, 255, 238));
    /// //assert_eq!(remaining, "");
    ///
    /// let result = hex_rgb.parse("#TEATEA");
    ///
    /// assert_eq!(result, Err(ParseError::NoMatch));
    ///
    /// # Ok::<(), ParseError>(())
    /// ```
    fn then<P: Parse>(self, parser: P) -> Then<Self, P>
    where
        Self: Sized,
    {
        then(self, parser)
    }
}

/// Functions that take &str and return `Result<(&str, &str), ParseError>` are Parseers.
///
/// The matched part of the input str is returned on the left hand side.
///
/// The remaining part of the input str is returned on the right hand side.
///
/// This is the same order that [`str::split_at()`] returns.
///
/// ```
/// use parsely::{digit, Parse, ParseError};
/// # use parsely::{char, hex, Lex, ParseResult};
///
/// fn my_parser(input: &str) -> Result<(u32, &str), ParseError> {
///     let boundary = input.find("abc").ok_or(ParseError::NoMatch)?;
///     let (_, remaining) = input.split_at(boundary + 3);
///
///     Ok((7, remaining))
/// }
///
/// // this parser function matches up to and including the token "abc", and outputs... 7
/// let (output, remaining) = my_parser("...abc")?;
/// assert_eq!(output, 7);
/// assert_eq!(remaining, "");
///
/// // assume we can use our hex_rgb parser from other examples
/// // use my_parser_lib::hex_rgb;
/// # fn hex_rgb<'i>(input: &'i str) -> ParseResult<'i, Rgb> {
/// #    let (_, remaining) = char('#').lex(input)?;
/// #    let hex_color = hex().count(2).map(|s| u8::from_str_radix(s, 16));
/// #
/// #    let (output, remaining) = hex_color.count(3).parse(remaining)?;
/// #    let mut colors = output.iter().copied();
/// #    let r = colors.next().ok_or(ParseError::NoMatch)?;
/// #    let g = colors.next().ok_or(ParseError::NoMatch)?;
/// #    let b = colors.next().ok_or(ParseError::NoMatch)?;
/// #
/// #    Ok((Rgb(r, g, b), remaining))
/// # };
/// # #[derive(PartialEq, Debug)]
/// # struct Rgb(u8, u8, u8);
///
/// // because it implements Parse, we can use it to build a more complex parser chain
/// let (output, remaining) = my_parser.then(hex_rgb).count(3).parse("...abc#AABBCC    abc#00FF00.. abc#FF00FF...")?;
/// let mut outputs = output.into_iter();
/// assert_eq!(outputs.next().unwrap(), (7, Rgb(170, 187, 204)));
/// assert_eq!(outputs.next().unwrap(), (7, Rgb(0, 255, 0)));
/// assert_eq!(outputs.next().unwrap(), (7, Rgb(255, 0, 255)));
/// assert!(outputs.next().is_none());
/// assert_eq!(remaining, "...");
///
/// # Ok::<(), ParseError>(())
/// ```
///
/// There is a type alias available to make the function signature *slightly* shorter
/// but it does need lifetime specifiers, we use `i` for input, the lifetime of the input str.
/// ```
/// use parsely::{digit, Parse, ParseError, ParseResult};
///
/// fn my_parser<'i>(input: &'i str) -> ParseResult<'i, u32> {
///    // ...
///    # let boundary = input.find("abc").ok_or(ParseError::NoMatch)?;
///    # let (_, remaining) = input.split_at(boundary + 3);
///    # Ok((7, remaining))
/// }
/// ```
impl<F, O> Parse for F
where
    F: Fn(&str) -> Result<(O, &str), ParseError>,
{
    type Output = O;

    fn parse<'i>(&mut self, input: &'i str) -> ParseResult<'i, O> {
        self(input)
    }
}
