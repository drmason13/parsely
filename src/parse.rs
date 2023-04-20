use std::ops::RangeBounds;

use crate::parser::combinator::{count, many, or, then, Many, Or, Then};

#[non_exhaustive]
#[derive(Debug, PartialEq)]
pub enum ParseError {
    NoMatch,
}

pub type ParseResult<'i> = Result<(&'i str, &'i str), ParseError>;

/// This trait is implemented by all Parsely parsers.
///
/// Its principle method is [`parse`](Parse::parse) which takes an input `&str` and returns an output, along with any remaining input.
///
/// ### Map parser output to a new type
///
/// The output of most parsers will be `&str`, the same type as the input.
///
//TODO: To map the output to a different type you can use the [`Parse::map`] or [`Parse::try_map`] methods which accept a closure to do the conversion.
///
//TODO: Some built in parsers accept a generic argument of a type to map the output to for you. For example [`parsers::int`] and [`parsers::number`].
pub trait Parse: Sized {
    /// The  method returns a tuple `(matched, remaining)` of `&str`.
    ///
    /// First the part of the input successfully matched and then the remaining part of the input that was not matched.
    ///
    /// The order reads left to right as the parser reads the input, and matches the return order of [`str::split_at`].
    fn parse<'i>(&mut self, input: &'i str) -> ParseResult<'i>;

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
    /// use parsely::{char, token, Parse, ParseError};
    ///
    /// let mut for_or_bar = token("foo").or(token("bar"));
    ///
    /// let (output, remaining) = for_or_bar.parse("foobarbaz")?;
    ///
    /// assert_eq!(output, "foo");
    /// assert_eq!(remaining, "barbaz");
    ///
    /// let (output, remaining) = for_or_bar.parse("barbaz")?;
    ///
    /// assert_eq!(output, "bar");
    /// assert_eq!(remaining, "baz");
    ///
    /// // `or` can be chained multiple times:
    ///
    /// let mut whitespace = char(' ')
    ///     .or(char('\t'))
    ///     .or(char('\n'))
    ///     .or(char('\r'));
    ///
    /// # Ok::<(), ParseError>(())
    /// ```
    ///
    /// Note that there is a whitespace parser available, see [`parsers::ws`]
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
    /// # Examples
    ///
    /// Basic usage:
    ///
    /// ```
    /// use parsely::{char, hex, Parse, ParseError};
    ///
    /// let mut hex_color = char('#').then(hex().many(1..));
    ///
    /// let (output, remaining) = hex_color.parse("#C0FFEE")?;
    ///
    /// assert_eq!(output, "#C0FFEE");
    /// assert_eq!(remaining, "");
    ///
    /// let result = hex_color.parse("#TEATEA");
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

/// Functions that take &str and return `Result<(&str, &str), ParseError>` are Parsers.
///
/// The matched part of the input str is returned on the left hand side.
///
/// The remaining part of the input str is returned on the right hand side.
///
/// This is the same order that [`str::split_at()`] returns.
///
/// ```
/// use parsely::{digit, Parse, ParseError};
///
/// fn my_parser(input: &str) -> Result<(&str, &str), ParseError> {
///     let boundary = input.find("abc").ok_or(ParseError::NoMatch)?;
///     let (output, remaining) = input.split_at(boundary + 3);
///
///     Ok((output, remaining))
/// }
///
/// // this parser function matches up to and including the token "abc"
/// let (output, remaining) = my_parser("...abc")?;
/// assert_eq!(output, "...abc");
///
/// // because it implements Parse, we can use it to build a more complex parser chain
/// let (output, remaining) = my_parser.then(digit().many(1..=3)).count(3).parse("...abc123.abc123..abc123...")?;
/// assert_eq!(output, "...abc123.abc123..abc123");
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
/// fn my_parser<'i>(input: &'i str) -> ParseResult<'i> {
///    // ...
///    # let boundary = input.find("abc").ok_or(ParseError::NoMatch)?;
///    # let (output, remaining) = input.split_at(boundary + 3);
///    # Ok((output, remaining))
/// }
/// ```
impl<F> Parse for F
where
    F: Fn(&str) -> Result<(&str, &str), ParseError>,
{
    fn parse<'i>(&mut self, input: &'i str) -> ParseResult<'i> {
        self(input)
    }
}
