use crate::{Lex, Parse};

/// This combinator is returned by [`pad()`]. See it's documentation for more details.
#[derive(Clone, Debug)]
pub struct Pad<L, R, T> {
    left: L,
    right: R,
    item: T,
}

impl<L, R, T> Parse for Pad<L, R, T>
where
    L: Lex,
    R: Lex,
    T: Parse,
{
    type Output = <T as Parse>::Output;

    fn parse<'i>(&self, input: &'i str) -> crate::ParseResult<'i, Self::Output> {
        let (_, remaining) = self.left.lex(input)?;
        let (output, remaining) = self.item.parse(remaining)?;
        let (_, remaining) = self.right.lex(remaining)?;

        Ok((output, remaining))
    }
}

/// Creates a parser that will lex with the left lexer, skipping the ouput, then parse with the parser, and then lex with the right lexer, skipping the ouput.
///
/// This serves to "pad" a parser allowing it to skip input on either side.
///
/// Note: This combinator violates the [Fundamental Law of Parsely Lexing] and has no method in the [`Lex`] trait.
///
/// Both the left and right lexer are required to match for the parse to be successful.
///
/// See [`Parse::pad()`] for more documentation and examples.
///
/// [Fundamental Law of Parsely Lexing]: crate::fundamental_law_of_parsely_lexing
pub fn pad<L: Lex, R: Lex, T>(left: L, right: R, item: T) -> Pad<L, R, T> {
    Pad { left, right, item }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::test_utils::*;
    use crate::{char, int};

    #[test]
    fn parsing() {
        test_parser_batch(
            ">int()< padding required",
            pad(char('>'), char('<'), int()),
            &[
                ("", None, ""), //
                (">123<", Some(123), ""),
                ("123<", None, "123<"),
                (">123", None, ">123"),
                (">>123<<", None, ">>123<<"),
            ],
        );
    }
}
