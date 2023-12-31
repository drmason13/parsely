use crate::Lex;

/// This lexer is returned by [`take()`]. See it's documentation for more details.
#[derive(Clone, Debug)]
pub struct Take {
    count: usize,
}

/// This lexer is returned by [`take_while()`]. See it's documentation for more details.
#[derive(Clone, Debug)]
pub struct TakeWhile<F> {
    condition: F,
}

impl Lex for Take {
    fn lex<'i>(&self, input: &'i str) -> crate::LexResult<'i> {
        if input.len() >= self.count {
            Ok(input.split_at(self.count))
        } else {
            Err(crate::InProgressError::no_match(input))
        }
    }
}

impl<F> Lex for TakeWhile<F>
where
    F: Fn(char) -> bool,
{
    fn lex<'i>(&self, input: &'i str) -> crate::LexResult<'i> {
        let char_indices = input.char_indices();
        let mut boundary = 0;

        for (i, c) in char_indices {
            if !(self.condition)(c) {
                break;
            } else {
                boundary = i + c.len_utf8();
            }
        }

        Ok(input.split_at(boundary))
    }
}

/// This lexer matches `count` characters if that many are available in the input.
///
/// If there are fewer than `count` characters in the input then this lexer fails.
pub fn take(count: usize) -> Take {
    Take { count }
}

/// This lexer matches all characters that satisfy the condition.
///
/// If no characters satisfy the condition, the lex is still successful.
///
/// # Examples
///
/// Basic usage:
///
/// ```
/// use parsely::{take_while, Lex};
///
/// let ascii_lexer = take_while(|c| c.is_ascii());
///
/// assert_eq!(ascii_lexer.lex("abc 123 $%^ â")?, ("abc 123 $%^ ", "â"));
/// assert_eq!(ascii_lexer.lex("abc 123 $%^ ẞ")?, ("abc 123 $%^ ", "ẞ"));
/// assert_eq!(ascii_lexer.lex("abc 123 $%^ ❤️")?, ("abc 123 $%^ ", "❤️"));
///
/// # Ok::<(), parsely::InProgressError>(())
/// ```
///
/// A more complex example:
///
/// ```
/// use parsely::{take_while, until, Lex};
///
/// let bang_or_question_mark = take_while(|c| c == '?' || c == '!');
///
/// let example = until(&['?', '!'][..]).then_skip(bang_or_question_mark);
///
/// assert_eq!(example.lex("what did you say?!?!?")?, ("what did you say", ""));
/// # Ok::<(), parsely::InProgressError>(())
/// ```
pub fn take_while<F>(condition: F) -> TakeWhile<F>
where
    F: Fn(char) -> bool,
{
    TakeWhile { condition }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::test_utils::test_lexer_batch;

    #[test]
    fn test_take_while() {
        test_lexer_batch(
            "take while alpha",
            take_while(char::is_alphabetic),
            &[
                ("abc", Some("abc"), ""),
                ("abc123", Some("abc"), "123"),
                // there's no minimum for take_while - perhaps we can add one?
                ("123abc", Some(""), "123abc"),
            ],
        );
    }
}
