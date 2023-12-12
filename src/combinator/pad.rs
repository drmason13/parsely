use crate::{result_ext::*, Lex, Parse};

/// This combinator is returned by [`pad()`]. See it's documentation for more details.
#[derive(Clone, Debug)]
pub struct Pad<L, R, T> {
    left: L,
    right: R,
    item: T,
}

impl<L: Lex, R: Lex, T> Lex for Pad<L, R, T>
where
    T: Lex,
{
    fn lex<'i>(&self, input: &'i str) -> crate::LexResult<'i> {
        let (_, inner) = self.left.lex(input)?;
        let (output, right_pad_and_more) = self.item.lex(inner).offset(input)?;
        let (_, remaining) = self.right.lex(right_pad_and_more).offset(input)?;

        Ok((output, remaining))
    }
}

impl<L, R, T> Parse for Pad<L, R, T>
where
    L: Lex,
    R: Lex,
    T: Parse,
{
    type Output = <T as Parse>::Output;

    fn parse<'i>(&self, input: &'i str) -> crate::ParseResult<'i, Self::Output> {
        let (_, inner) = self.left.lex(input)?;
        let (output, right_pad_and_more) = self.item.parse(inner).offset(input)?;
        let (_, remaining) = self.right.lex(right_pad_and_more).offset(input)?;

        Ok((output, remaining))
    }
}

/// Creates a parser that will lex with the left lexer, ignoring the ouput, then parse with the parser, and then lex with the right lexer, ignoring the ouput.
///
/// This serves to "pad" a parser allowing it to skip input on either side.
///
/// Both the left and right lexer are required to match for the parse to be successful.
///
/// See [`Parse::pad()`] for more documentation and examples.
pub fn pad<L: Lex, R: Lex, T>(left: L, right: R, item: T) -> Pad<L, R, T> {
    Pad { left, right, item }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::test_utils::*;
    use crate::{char, digit, int};

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

    #[test]
    fn lexing() {
        test_lexer_batch(
            ">digit()< padding required",
            pad(char('>'), char('<'), digit()),
            &[
                ("", None, ""), //
                (">1<", Some("1"), ""),
                ("2<", None, "2<"),
                (">3", None, ">3"),
                (">>4<<", None, ">>4<<"),
            ],
        );
    }
}
