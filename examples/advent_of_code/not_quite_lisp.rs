use parsely::{Lex, Parse};

// Parsers are defined and used inline in this function, it's quite concise!
pub fn find_floor(input: &str) -> Result<i32, parsely::Error> {
    let up = '('.map(|_| 1);
    let down = ')'.map(|_| -1);

    let (steps, _) = up.or(down).many(1..).parse(input)?;

    Ok(steps.iter().sum())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn examples_part1() -> Result<(), parsely::ErrorOwned> {
        assert!(
            find_floor("(())")? == 0 && find_floor("()()")? == 0,
            "(()) and ()() both result in floor 0."
        );

        assert!(
            find_floor("(((")? == 3 && find_floor("(()(()(")? == 3,
            "((( and (()(()( both result in floor 3."
        );

        assert_eq!(
            find_floor("))(((((")?,
            3,
            "))((((( also results in floor 3."
        );

        assert!(
            find_floor("())")? == -1 && find_floor("))(")? == -1,
            "()) and ))( both result in floor -1 (the first basement level)."
        );

        assert!(
            find_floor(")))")? == -3 && find_floor(")())())")? == -3,
            "))) and )())()) both result in floor -3."
        );

        Ok(())
    }
}
