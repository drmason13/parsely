use crate::Lex;

/// This lexer is returned by [`any()`]. See its documentation for more details.
#[derive(Debug, Clone)]
pub struct Any;

impl Lex for Any {
    fn lex<'i>(&self, input: &'i str) -> crate::LexResult<'i> {
        if let Some(c) = input.chars().next() {
            Ok(input.split_at(c.len_utf8()))
        } else {
            Err(crate::Error::no_match(input))
        }
    }
}

/// This parser will match and consume 1 char of the input.
///
/// If the input is empty then it fails.
pub fn any() -> Any {
    Any
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn any_works() {
        assert_eq!(any().lex("abc").unwrap(), ("a", "bc"));
    }

    #[test]
    fn any_works_with_unicode() {
        assert_eq!(any().lex("s❤️🧡💛💚💙💜").unwrap(), ("s", "❤️🧡💛💚💙💜"));

        // unicode is hard! unicode-segmentation would be needed to fix this.
        // note: \u{fe0f} is Unicode Variation selector 1 (i.e. the Red Heart emoji is the first variation of ❤)

        assert_eq!(
            any().lex("❤️🧡💛💚💙💜").unwrap(),
            ("❤", "\u{fe0f}🧡💛💚💙💜")
        );
        assert_eq!(
            any().lex("❤️t🧡💛💚💙💜").unwrap(),
            ("❤", "\u{fe0f}t🧡💛💚💙💜")
        );
    }
}
