use std::ops::RangeBounds;

use crate::{
    combinator::{
        count, many, map, optional, or, skip_then, then, try_map, Many, Map, Optional, Or,
        SkipThen, Then, TryMap,
    },
    Parse,
};

pub type LexResult<'i> = Result<(&'i str, &'i str), crate::Error>;

/// This trait is implemented by all Parsely lexers.
///
/// Its principle method is [`lex`](Lex::lex) which takes an input `&str` and returns the matched part of the input, along with any remaining unmatched input.
///
/// This is useful to break apart large complex input into smaller pieces which can be processed by parsers into other types.
pub trait Lex {
    /// This method returns a tuple `(matched, remaining)` of `&str`.
    ///
    /// First the part of the input successfully matched and then the remaining part of the input that was not matched.
    ///
    /// The order reads left to right as the lexer reads the input, and matches the return order of [`str::split_at`].
    fn lex<'i>(&self, input: &'i str) -> LexResult<'i>;

    /// Creates a new lexer that will attempt to lex with this lexer multiple times.
    ///
    /// See [`crate::combinator::Many`] for more details.
    fn many(self, range: impl RangeBounds<usize>) -> Many<Self>
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
    fn count(self, n: usize) -> Many<Self>
    where
        Self: Sized,
    {
        count(n, self)
    }

    /// Creates a new lexer that will match 0 or 1 times, making it optional.
    ///
    /// This is equivalent to `.many(0..=1)`.
    ///
    /// See [`crate::combinator::Many`] for more details.
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
    /// assert_eq!(result, Err(parsely::Error::NoMatch));
    ///
    /// # Ok::<(), parsely::Error>(())
    /// ```
    fn then<L: Lex>(self, lexer: L) -> Then<Self, L>
    where
        Self: Sized,
    {
        then(self, lexer)
    }

    /// Creates a parser that runs a parser on the remaining input after running this lexer.
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

    fn map<F, O>(self, f: F) -> Map<Self, F>
    where
        Self: Sized,
        F: Fn(&str) -> O,
    {
        map(self, f)
    }

    fn try_map<F, O, E>(self, f: F) -> TryMap<Self, F>
    where
        Self: Sized,
        F: Fn(&str) -> Result<O, E>,
    {
        try_map(self, f)
    }
}

/// Functions that take &str and return `Result<(&str, &str), parsely::Error>` are Lexers.
///
/// The matched part of the input str is returned on the left hand side.
///
/// The remaining part of the input str is returned on the right hand side.
///
/// This is the same order that [`str::split_at()`] returns.
///
/// ```
/// use parsely::{digit, Lex};
///
/// fn my_lexer(input: &str) -> Result<(&str, &str), parsely::Error> {
///     let boundary = input.find("abc").ok_or(parsely::Error::NoMatch)?;
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
/// There is a type alias available to make the function signature *slightly* shorter
/// but it does need lifetime specifiers, we use `i` for input, the lifetime of the input str.
/// ```
/// use parsely::{digit, Lex, LexResult};
///
/// fn my_lexer<'i>(input: &'i str) -> LexResult<'i> {
///    // ...
///    # let boundary = input.find("abc").ok_or(parsely::Error::NoMatch)?;
///    # let (output, remaining) = input.split_at(boundary + 3);
///    # Ok((output, remaining))
/// }
/// ```
impl<F> Lex for F
where
    F: Fn(&str) -> Result<(&str, &str), crate::Error>,
{
    fn lex<'i>(&self, input: &'i str) -> LexResult<'i> {
        self(input)
    }
}
