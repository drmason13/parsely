//! Parse JSON input somewhat according to spec.
//!
//! This isn't the fastest or most correct JSON parser out there, instead intended to demonstrate usage of parsely.

use std::{collections::BTreeMap, io::BufRead};

use parsely::{float, int, result_ext::*, ws, Lex, Parse, ParseResult};

// first come all the types we parse into...

/// All valid JSON can be represented as a single [`Value`].
#[derive(Debug, PartialEq)]
pub enum Value {
    Null,
    Bool(bool),
    Number(Number),
    String(String),
    Array(Vec<Value>),
    Object(Map<String, Value>),
}

/// Stores key value pairs
#[derive(Debug, PartialEq)]
pub struct Map<K, V>(BTreeMap<K, V>);

/// A float or integer
#[derive(Debug, PartialEq)]
pub struct Number(N);

// This strategy is inspired by serde_json
#[derive(Debug, PartialEq)]
pub enum N {
    Int(i64),
    Float(f64),
}

fn number() -> impl Parse<Output = Number> {
    float::<f64>()
        .map(|n| Number(N::Float(n)))
        .or(int::<i64>().map(|n| Number(N::Int(n))))
}

fn bool() -> impl Parse<Output = bool> {
    "true".map(|_| true).or("false".map(|_| false))
}

fn null() -> impl Parse<Output = Value> {
    "null".map(|_| Value::Null)
}

fn string() -> impl Parse<Output = String> {
    let str_inner = escape().many(..).or_until('"');

    str_inner.collect::<String>().pad_with('"', '"')
}

fn escape() -> impl Parse<Output = char> {
    parsely::escape(
        '\\',
        [
            ('\\', '\\'),
            ('t', '\t'),
            ('n', '\n'),
            ('r', '\r'),
            ('b', '\x08'),
            ('f', '\x0c'),
            ('"', '"'),
        ],
    )
}

// note that fn as parser is used here (and for map) because returning `impl Parse<Output = Vec<Value>>` would create a "recursive opaque type"
fn array(input: &str) -> ParseResult<'_, Vec<Value>> {
    parsely::combinator::pad('[', ']', value.many(..).delimiter(','.then(ws().many(..))))
        .parse(input)
        .offset(input)
}

fn map(input: &str) -> ParseResult<'_, Map<String, Value>> {
    parsely::combinator::pad(
        '{'.then(ws().many(..)),
        ws().many(..).then('}'),
        string()
            .then_skip(':'.pad())
            .then(value)
            .many(..)
            .delimiter(','.pad())
            .collect::<BTreeMap<String, Value>>(),
    )
    .map(Map)
    .parse(input)
    .offset(input)
}

fn value(input: &str) -> ParseResult<Value> {
    null()
        .or(bool().map(Value::Bool))
        .or(number().map(Value::Number))
        .or(string().map(Value::String))
        .or(array.map(Value::Array))
        .or(map.map(Value::Object))
        .pad()
        .parse(input)
}

fn json(input: &str) -> ParseResult<'_, Value> {
    value.parse(input).offset(input)
}

fn main() -> Result<(), parsely::ErrorOwned> {
    println!("Please enter some JSON to be parsed:");

    let stdin = std::io::stdin();
    let input = stdin.lock().lines().next().unwrap().unwrap();

    let (output, _remaining) = json(input.as_str())?;

    println!("{output:?}");

    Ok(())
}

impl std::fmt::Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Value::Null => write!(f, "null"),
            Value::Bool(b) => write!(f, "{b}"),
            Value::Number(n) => match n.0 {
                N::Int(n) => write!(f, "{n}"),
                N::Float(n) => write!(f, "{n}"),
            },
            Value::String(s) => write!(f, "\"{s}\""),
            Value::Array(a) => {
                write!(f, "[")?;
                for value in a {
                    write!(f, "{value},")?;
                }
                write!(f, "]")?;
                Ok(())
            }
            Value::Object(o) => {
                write!(f, "{{")?;
                for (key, value) in o.0.iter() {
                    write!(f, "\"{key}\": {value},")?;
                }

                write!(f, "}}")?;

                Ok(())
            }
        }
    }
}

#[cfg(test)]
mod json_tests {
    use super::*;

    #[test]
    fn arrays() -> Result<(), Box<dyn std::error::Error>> {
        assert_eq!(
            json("[1, 2, 3]")?,
            (
                Value::Array(vec![
                    Value::Number(Number(N::Int(1))),
                    Value::Number(Number(N::Int(2))),
                    Value::Number(Number(N::Int(3))),
                ]),
                ""
            )
        );

        // testing via display impl is easier to write tests for
        // note: while our display impl has its quirks (trailing commas mainly) it's good enough
        assert_eq!(format!("{}", json("[1, 2, 3]")?.0), r#"[1,2,3,]"#);

        assert_eq!(
            json("[[], [[]]]")?,
            (
                Value::Array(vec![
                    Value::Array(vec![]),
                    Value::Array(vec![Value::Array(vec![])]),
                ]),
                ""
            )
        );

        assert_eq!(format!("{}", json("[[], [[]]]")?.0), r#"[[],[[],],]"#);

        Ok(())
    }

    #[test]
    fn primitives() -> Result<(), Box<dyn std::error::Error>> {
        assert_eq!(json("1")?.0, Value::Number(Number(N::Int(1))));
        assert_eq!(json("123.45")?.0, Value::Number(Number(N::Float(123.45))));
        assert_eq!(
            json(r#""string""#)?.0,
            Value::String(String::from("string"))
        );
        assert_eq!(
            json(r#""string with:\tescapes\n""#)?.0,
            Value::String(String::from("string with:\tescapes\n"))
        );
        assert_eq!(json(r"true")?.0, Value::Bool(true));
        assert_eq!(json(r"false")?.0, Value::Bool(false));
        assert_eq!(json(r"null")?.0, Value::Null);

        Ok(())
    }

    #[test]
    fn escapes() -> Result<(), Box<dyn std::error::Error>> {
        assert_eq!(
            escape().parse(r#"\z"#),
            Err(parsely::Error::failed_conversion(r"\z"))
        );
        assert_eq!(escape().parse(r#"\""#)?, ('"', ""));
        assert_eq!(escape().parse(r#"\t"#)?, ('\t', ""));
        assert_eq!(escape().parse(r#"\n"#)?, ('\n', ""));
        assert_eq!(escape().parse(r#"\r"#)?, ('\r', ""));
        assert_eq!(escape().parse(r#"\b"#)?, ('\x08', ""));
        assert_eq!(escape().parse(r#"\f"#)?, ('\x0c', ""));
        assert_eq!(escape().parse(r#"\\"#)?, ('\\', ""));

        assert_eq!(
            json(r#""\z""#),
            Err(parsely::Error::no_match(r#"\z""#).offset(r#""\z""#))
        );
        assert_eq!(json(r#""\"""#)?.0, Value::String(String::from("\"")));
        assert_eq!(json(r#""\n""#)?.0, Value::String(String::from("\n")));
        assert_eq!(json(r#""\\""#)?.0, Value::String(String::from("\\")));

        Ok(())
    }

    #[test]
    fn maps() -> Result<(), Box<dyn std::error::Error>> {
        assert_eq!(
            format!("{}", json(r#"{"foo": "bar"}"#)?.0),
            r#"{"foo": "bar",}"#
        );

        assert_eq!(
            format!("{}", json(r#"{"foo": ["bar"]}"#)?.0),
            r#"{"foo": ["bar",],}"#
        );

        assert_eq!(
            format!("{}", json(r#"{"foo": ["bar", {"baz": 123}]}"#)?.0),
            r#"{"foo": ["bar",{"baz": 123,},],}"#
        );

        assert_eq!(
            format!(
                "{}",
                json(r#"[[{"foo": "bar"}], [[null, true, false, 1], 2],   {"7": 7}]"#)?.0
            ),
            r#"[[{"foo": "bar",},],[[null,true,false,1,],2,],{"7": 7,},]"#
        );

        assert_eq!(
            format!(
                "{}",
                json(r#"{"one": 1, "two": 2, "three": 3, "foo": "bar"}"#)?.0
            ),
            r#"{"foo": "bar","one": 1,"three": 3,"two": 2,}"#
        );

        Ok(())
    }

    #[test]
    fn whitespace() -> Result<(), Box<dyn std::error::Error>> {
        assert_eq!(
            format!("{}", json("{ \"foo\" \t \n :  \"bar\"\t}")?.0),
            r#"{"foo": "bar",}"#,
            "spaces should be ignored"
        );

        assert_eq!(
            format!("{}", json("{ \"foo\" \t \n :  \"bar\"\t}")?.0),
            r#"{"foo": "bar",}"#,
            "other whitespace should also be ignored"
        );

        assert_eq!(
            format!(
                "{}",
                json("[[{\"foo\": \"bar\"}]\t,  [\t[ null \n\t,  true,false,    1 \t ]\t,\t 2], \t  {\"7\": 7}]")?.0
            ),
            r#"[[{"foo": "bar",},],[[null,true,false,1,],2,],{"7": 7,},]"#,
            "nested arrays and objects should also ignore any whitespace"
        );

        Ok(())
    }

    #[test]
    fn it_works() -> Result<(), Box<dyn std::error::Error>> {
        assert_eq!(
            format!("{}", json(r#"{"foo": 123, "poop": [{"x":"x"},2,3]}"#)?.0),
            r#"{"foo": 123,"poop": [{"x": "x",},2,3,],}"#,
            "this json is valid so it should parse"
        );

        Ok(())
    }
}
