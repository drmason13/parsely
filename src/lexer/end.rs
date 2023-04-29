use crate::{Lex, LexResult};

/// This lexer is returned by [`end()`]. See it's documentation for more details.
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

/// Matches the end of input, i.e. if the input is empty.
///
/// # Examples
///
/// Basic usage:
///
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
///
/// You should be careful using this lexer because it can cause matching to fail when it is reused:
///
/// ```
/// use parsely::{char, end, Lex};
///
/// let lexer = char('a').count(3).then(end());
///
/// // this can't ever match because lexer expects the input to end after the first match
/// let lexer_multi = lexer.many(2..);
///
/// let result = lexer_multi.lex("aaaaaa");
/// assert_eq!(result, Err(parsely::Error::NoMatch));
/// # Ok::<(), parsely::Error>(())
/// ```
pub fn end() -> End {
    End
}
