use crate::{Lex, LexError, LexResult};

pub struct End;

impl Lex for End {
    fn lex<'i>(&mut self, input: &'i str) -> LexResult<'i> {
        if input.is_empty() {
            Ok(("", ""))
        } else {
            Err(LexError::NoMatch)
        }
    }
}

/// ```
/// use parsely::{char, end, Lex, LexError};
///
/// let mut lexer = char('a').count(3).then(end());
///
/// let (output, remaining) = lexer.lex("aaa")?;
///
/// assert_eq!(output, "aaa");
/// assert_eq!(remaining, "");
///
///
/// let result = lexer.lex("aaaaaaaaa");
/// assert_eq!(result, Err(LexError::NoMatch));
///
/// # Ok::<(), LexError>(())
/// ```
pub fn end() -> End {
    End
}
