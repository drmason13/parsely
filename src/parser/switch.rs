use crate::{token, Error, Lex, Parse};

pub struct Switch<L, T, const N: usize> {
    items: [(L, T); N],
}

/// Creates a Switch parser that parses input by trying each provided lexer in turn and mapping them to the corresponding output type
///
/// Requirements:
///
/// * Each output must be the same type, and that is the type this parser produces
/// * Each lexer must be provided in order they are to be attempted to match
/// * The output type must be Clone as one will be cloned ecah time the parser succeeds
///
/// Typical usage is to map a series of tokens 1:1 with a corresponding Rust type, which can be done more verbosely by using a lot of token, map and or:
///
/// ```
/// use parsely::{Lex, Parse, token};
///
/// #[derive(Debug, PartialEq, Clone, Copy)]
/// pub enum MyTokens {
///     Foo,
///     Bar,
///     Baz,
///     Quux,
/// }
/// # fn main() -> Result<(), parsely::Error> {
/// let my_token_parser = token("foo").map(|_| MyTokens::Foo)
///     .or(token("bar").map(|_| MyTokens::Bar))
///     .or(token("baz").map(|_| MyTokens::Baz))
///     .or(token("quux").map(|_| MyTokens::Quux));
///
/// assert_eq!(my_token_parser.parse("foo 123")?, (MyTokens::Foo, " 123"));
/// # Ok(()) }
/// ```
///
/// The above is simplified by using `switch()` (note that str literals are accepted without token(), this is a special case for switch for convenience):
/// ```
/// use parsely::{Lex, Parse, switch};
///
/// #[derive(Debug, PartialEq, Clone, Copy)]
/// pub enum MyTokens {
///     Foo,
///     Bar,
///     Baz,
///     Quux,
/// }
/// # fn main() -> Result<(), parsely::Error> {
/// let my_token_parser = switch([
///     ("foo", MyTokens::Foo),
///     ("bar", MyTokens::Bar),
///     ("baz", MyTokens::Baz),
///     ("quux", MyTokens::Quux),
/// ]);
///
/// assert_eq!(my_token_parser.parse("foo 123")?, (MyTokens::Foo, " 123"));
/// #  Ok(()) }
/// ```
pub fn switch<L, T, const N: usize>(items: [(L, T); N]) -> Switch<L, T, N> {
    Switch { items }
}

impl<L, T, const N: usize> Parse for Switch<L, T, N>
where
    L: Lex,
    T: Clone,
{
    type Output = T;

    fn parse<'i>(&self, input: &'i str) -> crate::ParseResult<'i, Self::Output> {
        for (lexer, output) in self.items.iter() {
            if let Ok((_, remaining)) = lexer.lex(input) {
                return Ok((output.clone(), remaining));
            }
        }
        Err(Error::NoMatch)
    }
}

impl<T, const N: usize> Parse for Switch<&'static str, T, N>
where
    T: Clone,
{
    type Output = T;

    fn parse<'i>(&self, input: &'i str) -> crate::ParseResult<'i, Self::Output> {
        for (s, output) in self.items.iter() {
            let lexer = token(s);
            if let Ok((_, remaining)) = lexer.lex(input) {
                return Ok((output.clone(), remaining));
            }
        }
        Err(Error::NoMatch)
    }
}
