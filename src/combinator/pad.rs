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
/// # Combining pad with then
///
/// Due to pad() skipping the remainder, when used inside then in a lexer, the skipped remainder **is not skipped**!
///
/// ```
/// use parsely::{digit, Lex};
///
/// let lexer = digit().count(2).then(":".pad());
/// let input = "99:\r\n\r\n";
///
/// let (matched, remaining) = lexer.lex(input)?;
///
/// assert_eq!(matched, "99:");
/// // pad didn't skip any whitespace in our lexer!
/// assert_eq!(remaining, "\r\n\r\n");
///
/// // the solution is to reorder our lexer:
/// let fixed_lexer = digit().count(2).then(":").pad();
/// //                                         ^^ pad the result of .then()
///
/// let (matched, remaining) = fixed_lexer.lex(input)?;
/// assert_eq!(matched, "99:");
/// assert_eq!(remaining, "");
/// # Ok::<(), parsely::Error>(())
/// ```
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
    fn pad_works() -> Result<(), crate::ErrorOwned> {
        let lexer = ":".pad();
        let input = ":\r\n\r\n";
        let (matched, remaining) = lexer.lex(input)?;
        assert_eq!(matched, ":");
        assert_eq!(remaining, "");
        Ok(())
    }

    #[test]
    fn pad_sometimes_does_not_work() -> Result<(), crate::ErrorOwned> {
        let lexer = "foo".then(":".pad());
        let input = "foo:\r\n\r\n";

        let (matched, remaining) = lexer.lex(input)?;
        assert_eq!(matched, "foo:");
        // This is a surprising consequence of how Then implements Lex
        assert_eq!(remaining, "\r\n\r\n");
        Ok(())
    }

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
