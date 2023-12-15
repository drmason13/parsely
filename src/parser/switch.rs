use crate::{Error, Lex, Parse};

pub struct Switch<L, T, const N: usize> {
    items: [(L, T); N],
}

/// Creates a Switch parser that parses input by trying each provided lexer in turn and mapping them to the corresponding output.
///
/// Note that for convenience, `"string literals"` can be used in place of `token("string literals")`.
///
/// Requirements:
///
/// * **The output type must impl `Clone`** as the matched output will be cloned each time the parser succeeds[^why]
/// * Each output must be the same type, and that is the type this parser produces
/// * Each lexer must be provided in order they are to be attempted to match
///
/// Typical usage is to map a series of tokens 1:1 with a corresponding Rust type, which can be done more verbosely by using a lot of token, map and or:
///
/// [^why]: this allows the parser itself to own the inputs and outputs and be reused to parse multiple different inputs.
/// This shouldn't differ in performance to the manual `.or().map()` version since that version will create a new value every time instead of cloning.
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
///
/// let my_token_parser = "foo".map(|_| MyTokens::Foo)
///     .or("bar".map(|_| MyTokens::Bar))
///     .or("baz".map(|_| MyTokens::Baz))
///     .or("quux".map(|_| MyTokens::Quux));
///
/// assert_eq!(my_token_parser.parse("foo 123")?, (MyTokens::Foo, " 123"));
/// # Ok::<(), parsely::Error>(())
/// ```
///
/// The above is simplified by using `switch()`
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
///
/// let my_token_parser = switch([
///     ("foo", MyTokens::Foo),
///     ("bar", MyTokens::Bar),
///     ("baz", MyTokens::Baz),
///     ("quux", MyTokens::Quux),
/// ]);
///
/// assert_eq!(my_token_parser.parse("foo 123")?, (MyTokens::Foo, " 123"));
/// # Ok::<(), parsely::Error>(())
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
