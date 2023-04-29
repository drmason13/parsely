use crate::{Lex, LexResult};

#[derive(Debug, Clone)]
pub struct End;

impl Lex for End {
    fn lex<'i>(&self, input: &'i str) -> LexResult<'i> {
        if input.is_empty() {
            Ok(("", ""))
        } else {
            Err(crate::Error::NoMatch)
        }
    }
}

/// ```
/// use parsely::{char, end, Lex};
///
/// let lexer = char('a').count(3).then(end());
///
/// let (output, remaining) = lexer.lex("aaa")?;
///
/// assert_eq!(output, "aaa");
/// assert_eq!(remaining, "");
///
///
/// let result = lexer.lex("aaaaaaaaa");
/// assert_eq!(result, Err(parsely::Error::NoMatch));
///
/// # Ok::<(), parsely::Error>(())
/// ```
pub fn end() -> End {
    End
}
