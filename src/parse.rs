use std::ops::RangeBounds;

use crate::parser::combinator::{count, many, or, then, Many, Or, Then};

#[non_exhaustive]
#[derive(Debug, PartialEq)]
pub enum ParseError {
    NoMatch,
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
    fn many(self, range: impl RangeBounds<usize>) -> Many<Self, Self::Output>
    where
        Self: Sized,
    {
        many(range, self)
    }

    /// Creates a new parser that will attempt to parse with this parser exactly n times.
    ///
    /// See [`combinators::Many`] for more details.
    fn count(self, n: usize) -> Many<Self, Self::Output>
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
    fn or<P: Parse, O>(self, parser: P) -> Or<Self, P, O>
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
    fn then<P: Parse, O>(self, parser: P) -> Then<Self, P, O>
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
/// // because it implements Parse, we can use it to build a more compparse parser chain
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
/// fn my_parser<'i>(input: &'i str) -> ParseResult<'i, O> {
///    // ...
///    # let boundary = input.find("abc").ok_or(ParseError::NoMatch)?;
///    # let (output, remaining) = input.split_at(boundary + 3);
///    # Ok((output, remaining))
/// }
/// ```
impl<F, O> Parse for F
where
    F: Fn(&str) -> Result<(&str, O), ParseError>,
{
    fn parse<'i>(&mut self, input: &'i str) -> ParseResult<'i, O> {
        self(input)
    }
}
