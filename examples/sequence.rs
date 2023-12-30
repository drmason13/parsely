//! This example demonstrates some parsers that match multiple items sequentially and escape certain characters.
//!
//! The code is heavily commented, explaining what each part is doing.
//!
//! We'll build a few CSV parsers, starting simple and building in complexity.
//!
//! Each Comma Separated Variable (CSV) parser will parse multiple values separated by commas.
//!
//! e.g.
//!
//! ```
//! 1,2,3,4,5
//! ```
//! is how you'd write the numbers 1 to 5 in csv.
//!
//! The most important combinator used here is `many()` which takes a range to describe how many times a parser/lexer should match.
//!
//! Also of note is the `.delimiter()` and the `.or_until()` methods of `many()`, as well as the `escape()` built in parser.

use parsely::{any, end, int, none_of, Lex, Parse, ParseResult};

// This csv parser will output a sequence of integers
//
// The return type basically means "return some type that Parses a Vec<i32>" - this avoids having to name the type which would be difficult!
fn number_csv() -> impl Parse<Output = Vec<i32>> {
    // int is a built in parsely parser
    int::<i32>()
        // .many(..) builds a `many` combinator out of the `int` parser
        // the range .. means any number of matches (including 0) are accepted by the `many` combinator
        .many(..)
        // delimiter(',') is a method of the `many` combinator
        // it makes the `many` combinator **lex** a comma character ',' after each match.
        // If any comma is missing between values, it won't match.
        // Fortunately the trailing match is optional.
        .delimiter(',')
}

// This csv parser will output a sequence of strings
//
// It will have a flaw though: the strings won't be able to contain commas.
// Every time we see a comma, we'll stop parsing one value and start parsing the next
fn string_csv() -> impl Parse<Output = Vec<String>> {
    // none_of is a built in parsely lexer - it matches any char that isn't in the given str
    // we use it here to match anything except a comma
    let value = none_of(",")
        // here we collect at least 1 char worth of input
        // we aren't outputting a Vec because none_of is a lexer not a parser, many will simply keep lexing until it fails and then split the input str at that point
        .many(1..)
        // we use map to transform the **lexer** into a parser. We'll turn each &str slice into a String
        // note, this is the same as doing `.map(|s| s.to_string())` but neater
        .map(str::to_string);

    // now we can parse a single value, we can parse any number of them separated by commas in the usual way
    value.many(..).delimiter(',')
}

// This csv parser will output a sequence of strings but this time the strings may contain commas, we enable this by using an **escape sequence**:
//
// The escape sequence is a built in parsely parser (you could build your own quite easily though, check out the source code of paresly::escape)
//
// It takes an escape character, and an array of escape sequences - that is the character being escaped and the character it should be transformed into
fn escaped_csv() -> impl Parse<Output = Vec<String>> {
    let value = parsely::escape(
        // the escape character is \ - note: in rust source code \ itself needs escaping with a \
        '\\',
        [
            // we transform any \, into ,
            (',', ','),
            // we transform any \n into an actual newline character
            ('n', '\n'),
            // and so on...
            ('r', '\r'),
            ('t', '\t'),
            // we also allow escaping \ so our strings may contain actual \ characters
            ('\\', '\\'),
        ],
    )
    // the escape sequence parser only returns a single character, let's parse 1 or more in a row
    .many(1..)
    // many *defaults* to returning a `Vec` for convenience, but it's capable of collecting into any type that implements `std::iter::Extend`
    //
    // .collect() is a method to change the type that many collects into from a Vec into something else
    // here we are telling the many to collect the chars into a String instead of a Vec
    .collect::<String>()
    // the many parser has no max bound, so it would attempt to convert the whole csv into one String - that's no good!
    // the escape sequence would eat the commas!
    // To avoid that we use another method on the `many` combinator: `or_until()`
    // or_until(',') makes the many stop parsing if it "sees" a ','.
    // It "sees" the comma by attempting to lex one after each match, if the lex succeeds then it sees it and stops
    .or_until(',');

    // now we can parse a single value, we can parse any number of them separated by commas in the usual way
    value.many(..).delimiter(',')
}

// This csv parser will output a sequence of strings
//
// This time we'll do what a lot of csv parsers do and allow values to contain commas if they are quoted - that is preceeded and followed by a quote char
//
// in order to allow strings to contain literal quotes, we'll transform repeated double quotes into one double quote inside of double quotes.
// these escaping rules don't match those of parsely's built in escape parser, so we'll build our own using a function
fn quote_csv() -> impl Parse<Output = Vec<String>> {
    let value = none_of("\"").many(1..).or_until(",").map(str::to_string);

    let string = '"' // start a string with a double quote
        // we *match* the double quote but we don't want it to be part of our string output, so we *skip* it, *then* parse something, using `.skip_then()`
        .skip_then(
            // this is our custom escape parser, we define it in the function below this one
            repeat_quote_escape
                .many(1..)
                // the condition to stop is a bit more complicated this time
                // (don't worry, nothing we pass to or_until() *consumes* any input)
                .or_until(
                    // We want to stop as soon as we see the closing "
                    "\""
                    // but *then* we want to be sure that we're actually at the end of a value
                    // if not, we should fail since that means it must have been a lone " in the middle of a value - which is not allowed!
                    .then(
                        // we do that by checking the " was followed by a comma or the end of the input
                        // (which is exactly what the built in parsely lexer `end()` does)
                        ','.or(end())
                    )
                )
                // since our custom parser outputs chars just like the built in one, we can collect into a String as usual
                .collect::<String>(),
        )
        // we consume the closing double quote, but not include it in our string output
        // so after our parser, we *then* *skip* some lexer, in this case just the double quote
        .then_skip('"');

    // we have two ways of parsing a value this time: a quoted one (a.k.a string)
    string.or(
        // or an unquoted one (a.k.a value)
        value
    )
    // the repeating and separating by commas is the same as usual
    .many(..).delimiter(',')
}

// this is our custom escape function parser
fn repeat_quote_escape(input: &str) -> ParseResult<char> {
    // see if the input starts with double (double)quotes: ""
    match "\"\"".lex(input) {
        // Ok means it matched (and does start with double quotes)
        Ok(
            // ignore the double quotes we just matched for now, but keep the remaining part of the input
            (_, remaining),
        ) => Ok(
            // return one double quote character - we've just transformed two quotes into one,
            ('"', remaining), // don't forget to include the remaining input
        ),

        // Err means it did not match (and does *not* start with double quotes)
        Err(_) => {
            // no double quotes means we can just return whatever char is next
            let (matched, remaining) = 
                // which is exactly what the built in parsely lexer `any()` does:
                any().lex(input)?;  // this ? means we'll return an error and not match if the input is empty
            // but `any()` doesn't return a `char` for us, it returns `&str` containing one char
            Ok((
                // unwrap: any fails if the input is empty
                matched.chars().next().unwrap(),  // this is our output, the next char
                remaining,  // don't forget to include the remaining input
            ))
        }
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // running number_csv() produces the parser, we have to call `.parse(input)` to do the actual parsing
    let (output, remaining) = number_csv().parse("1,2,3,4,5")?;

    assert_eq!(output, vec![1, 2, 3, 4, 5]);
    // remaining is an empty &str meaning that the whole input was matched, good!
    assert_eq!(remaining, "");

    let (output, remaining) = string_csv().parse(r"foo,bar,baz, quux, and so on")?;

    assert_eq!(
        output,
        vec![
            String::from("foo"),
            String::from("bar"),
            String::from("baz"),
            String::from(" quux"),
            String::from(" and so on"),
        ],
    );
    assert_eq!(remaining, "");

    let (output, remaining) = escaped_csv()
        .parse(r"foo,bar,baz\, quux\, and so on,note how the \\\, escape worked\npretty neat huh?")?;

    assert_eq!(
        output,
        vec![
            String::from("foo"),
            String::from("bar"),
            String::from("baz, quux, and so on"),
            String::from(
                "note how the \\, escape worked
pretty neat huh?"
            ),
        ]
    );
    assert_eq!(remaining, "");

    let (output, remaining) = quote_csv().parse(
        r#"foo,bar,"baz, quux, and so on","note how the ""quoted value"" worked\npretty neat huh?""#,
    )?;

    assert_eq!(
        output,
        vec![
            String::from("foo"),
            String::from("bar"),
            String::from("baz, quux, and so on"),
            // the newline character isn't escaped.
            // As an exercise for the reader:
            //
            // Can you write a parser that parses quoted csv but also transforms escape sequences like \n, \r and \t into real newline characters and tabs?
            String::from(r#"note how the "quoted value" worked\npretty neat huh?"#),
        ]
    );
    assert_eq!(remaining, "");

    // some more quoted csv tests from https://stackoverflow.com/questions/4617935/is-there-a-way-to-include-commas-in-csv-columns-without-breaking-the-formatting

    // "Fresh, brown ""eggs""" -> Fresh, brown "eggs"
    assert_eq!(
        quote_csv().parse(r#""Fresh, brown ""eggs""""#)?.0[0],
        r#"Fresh, brown "eggs""#
    );

    // """" -> "
    assert_eq!(quote_csv().parse(r#""""""#)?.0[0], r#"""#);

    // """,""" -> ","
    assert_eq!(quote_csv().parse(r#"""",""""#)?.0[0], r#"",""#);

    // ",,,""" -> ,,,"
    assert_eq!(quote_csv().parse(r#"",,,""""#)?.0[0], r#",,,""#);

    // ",""""," -> ,"",
    assert_eq!(quote_csv().parse(r#"","""",""#)?.0[0], r#","","#);

    // """""""" -> """
    assert_eq!(quote_csv().parse(r#""""""""""#)?.0[0], r#"""""#);

    Ok(())
}
