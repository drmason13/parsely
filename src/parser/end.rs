use crate::{Parse, ParseError, ParseResult};

pub struct End;

impl Parse for End {
    fn parse<'i>(&mut self, input: &'i str) -> ParseResult<'i> {
        if input.is_empty() {
            Ok(("", ""))
        } else {
            Err(ParseError::NoMatch)
        }
    }
}

/// ```
/// use parsely::{char, end, Parse, ParseError};
///
/// let mut parser = char('a').count(3).then(end());
///
/// let (output, remaining) = parser.parse("aaa")?;
///
/// assert_eq!(output, "aaa");
/// assert_eq!(remaining, "");
///
///
/// let result = parser.parse("aaaaaaaaa");
/// assert_eq!(result, Err(ParseError::NoMatch));
///
/// # Ok::<(), ParseError>(())
/// ```
pub fn end() -> End {
    End
}
