use crate::Lex;

#[derive(Clone, Debug)]
pub struct Take {
    count: usize,
}

#[derive(Clone, Debug)]
pub struct TakeWhile<F> {
    condition: F,
}

impl Lex for Take {
    fn lex<'i>(&mut self, input: &'i str) -> crate::LexResult<'i> {
        if input.len() >= self.count {
            Ok(input.split_at(self.count))
        } else {
            Err(crate::Error::NoMatch)
        }
    }
}

impl<F> Lex for TakeWhile<F>
where
    F: Fn(char) -> bool,
{
    fn lex<'i>(&mut self, input: &'i str) -> crate::LexResult<'i> {
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

pub fn take(count: usize) -> Take {
    Take { count }
}

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
