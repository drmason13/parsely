use crate::Lex;

use self::pattern::PatternLite;

mod pattern;

/// This lexer is returned by [`until()`]. See it's documentation for more details.
#[derive(Clone)]
pub struct Until<'a> {
    pattern: PatternLite<'a>,
}

impl<'a> Lex for Until<'a> {
    fn lex<'i>(&self, input: &'i str) -> crate::LexResult<'i> {
        let found_index = match &self.pattern {
            PatternLite::Char(x) => input.find(*x),
            PatternLite::Str(x) => input.find(x),
            PatternLite::CharSlice(x) => input.find(*x),
        };

        match found_index {
            Some(boundary) => Ok(input.split_at(boundary)),
            None => Err(crate::Error::NoMatch),
        }
    }
}

/// Creates a lexer that matches all characters up until (but not including) `pattern`.
///
/// The pattern can be a [`&str`](prim@str), [`char`], or a slice of [`char`]s.
///
/// # Examples
///
/// Basic usage:
///
/// ```
/// use parsely::{until, Lex};
///
/// let until_a = until('a');
///
/// assert_eq!(until_a.lex("abcdef")?, ("", "abcdef"));
/// assert_eq!(until_a.lex("fedcba")?, ("fedcb", "a"));
///
///
/// let until_def = until("def");
///
/// assert_eq!(until_def.lex("abcdef")?, ("abc", "def"));
/// assert_eq!(until_def.lex("fedcba"), Err(parsely::Error::NoMatch));
///
///
/// let until_abc_slice = until(&['a', 'b', 'c'][..]);
///
/// assert_eq!(until_abc_slice.lex("abcdef")?, ("", "abcdef"));
/// assert_eq!(until_abc_slice.lex("fedcba")?, ("fed", "cba"));
///
/// # Ok::<(), parsely::Error>(())
/// ```
///
///
pub fn until<'a, P>(pattern: P) -> Until<'a>
where
    P: Into<PatternLite<'a>> + Clone,
{
    Until {
        pattern: pattern.into(),
    }
}
