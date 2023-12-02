use crate::{Error, Lex, LexResult, Parse, ParseResult};

/// This combinator is returned by [`crawl()`]. See it’s documentation for more details
pub struct Crawl<T> {
    item: T,
}

impl<P> Parse for Crawl<P>
where
    P: Parse,
{
    type Output = <P as Parse>::Output;

    fn parse<'i>(&self, input: &'i str) -> ParseResult<'i, Self::Output> {
        let mut char_indices = input.char_indices();
        let Some((mut boundary, _)) = char_indices.next() else {
            return Err(Error::NoMatch);
        };

        loop {
            if let Ok((matched, _)) = self.item.parse(&input[boundary..]) {
                boundary = match char_indices.next() {
                    Some((n, _)) => n,
                    None => input.len(),
                };
                return Ok((matched, &input[boundary..]));
            } else if boundary == input.len() {
                return Err(Error::NoMatch);
            } else {
                boundary = match char_indices.next() {
                    Some((n, _)) => n,
                    None => input.len(),
                };
            }
        }
    }
}

impl<L> Lex for Crawl<L>
where
    L: Lex,
{
    fn lex<'i>(&self, input: &'i str) -> LexResult<'i> {
        let mut char_indices = input.char_indices();
        let Some((mut boundary, _)) = char_indices.next() else {
            return Err(Error::NoMatch);
        };

        loop {
            if let Ok((matched, _)) = self.item.lex(&input[boundary..]) {
                boundary = match char_indices.next() {
                    Some((n, _)) => n,
                    None => input.len(),
                };
                return Ok((matched, &input[boundary..]));
            } else if boundary == input.len() {
                return Err(Error::NoMatch);
            } else {
                boundary = match char_indices.next() {
                    Some((n, _)) => n,
                    None => input.len(),
                };
            }
        }
    }
}

/// Crawls through the input one char at a time.
///
/// Usually the input is consumed by an amount equal to what was matched.
/// Crawl does not do this; instead the input is always advanced by a single character, regardless of the length of the match.
///
/// This makes it a very unusual combinator!
///
/// Crawl only fails to match after all of the input has been seen without finding any match.
///
/// Crawl is not a sequence combinator, so only one result is returned.
///
/// Crawl's behavior is *like* that of regular expressions, but since input is only partially consumed, crawl supports matching all[^all] overlapping matches.
///
/// [^all] The input is always advanced by one char in order to avoid infinite loops, this means that not *all* overlapping matches are made: only one match per start index.
///
/// # Examples
///
/// Match a token anywhere inside a string.
///
/// ```
/// use parsely::{combinator::crawl, token, Lex};
///
/// let input = "bla bla bla >>>abc<<< and so on...";
///
/// let (matched, remaining) = crawl(token("abc")).lex(input)?;
/// assert_eq!(matched, "abc");
///
/// // NOTE: only one char is consumed even though 3 chars were matched!
/// assert_eq!(remaining, "bc<<< and so on...");
///
/// let (matched, remaining) = token("abc").optional().lex(input)?;
/// assert_eq!(matched, "");
/// assert_eq!(remaining, input);
///
/// # Ok::<(), parsely::Error>(())
/// ```
///
/// Crawl is often combined with [`Many`](crate::combinator::many) to find all **overlapping** matches.
///
/// Just be careful to crawl and then many -> `crawl(your_parser).many(..)` and not the other way around!
/// ```
/// use parsely::{combinator::crawl, token, Lex, Parse};
///
/// // we'd like to match both "two" and "one", even though `o` is part of both matches
/// let input = "twone";
///
/// let one_or_two = || token("one").map(|_| 1)
///     .or(token("two").map(|_| 2));
///
/// let (matched, remaining) = crawl(one_or_two()).many(..).parse(input)?;
/// assert_eq!(&matched, &[2, 1][..]);
/// assert_eq!(remaining, "ne");
///
/// // Helpfully or not, intervening input is ignored!
/// let input = "twone pilots ➔ uh, I mean *twentyone* pilots!";
///
/// let (matched, remaining) = crawl(one_or_two()).many(..).parse(input)?;
/// assert_eq!(&matched, &[2, 1, 1][..]);
/// assert_eq!(remaining, "ne* pilots!");
///
/// # Ok::<(), parsely::Error>(())
/// ```
pub fn crawl<T>(item: T) -> Crawl<T> {
    Crawl { item }
}
