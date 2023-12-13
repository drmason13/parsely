#[cfg(test)]
mod tests {
    use parsely::*;
    #[test]
    fn test_pad() -> Result<(), parsely::Error> {
        let parser = token("foo")
            .map(str::to_string)
            .pad()
            .then(token("bar").map(str::to_string));

        assert_eq!(
            parser.parse("foo   bar")?,
            (("foo".to_string(), "bar".to_string()), "")
        );

        // if we want the string "foobar" we would need to do something inefficient to join two Strings
        let parser = token("foo")
            .map(str::to_string)
            .pad()
            .then(token("bar").map(str::to_string))
            .map(|(a, b)| format!("{a}{b}"));

        assert_eq!(parser.parse("foo   bar")?, (("foobar".to_string()), ""));

        Ok(())
    }
}

fn main() {}
