use crate::Lex;

#[derive(Debug, Clone)]
pub struct Match<T> {
    item: T,
    count: usize,
}

#[derive(Debug, Clone)]
pub struct MatchWhile<T, F> {
    item: T,
    condition: F,
}

impl Lex for Match<L>
where
    L: Lex,
{
    fn lex<'i>(&mut self, input: &'i str) -> crate::LexResult<'i> {
        if input.len() >= self.count {
            Ok(input.split_at(self.count))
        } else {
            Err(crate::Error::NoMatch)
        }
    }
}

impl<T, F> Lex for MatchWhile<T, F>
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
                boundary = i;
            }
        }

        Ok(input.split_at(boundary))
    }
}
