//! Parse JSON input somewhat according to spec.
//!
//! This isn't the fastest or most correct JSON parser out there, instead intended to demonstrate usage of parsely.

// first come all the types we parse into...

use std::{
    collections::BTreeMap,
    io::BufRead,
    num::{ParseFloatError, ParseIntError},
};

use parsely::{char, ws, Lex, Parse, ParseResult};

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

// This strategy is inspired by (shamelessly copied from) serde_json
#[derive(Debug, PartialEq)]
pub enum N {
    Int(i64),
    Float(f64),
}

pub enum JsonParseError {
    InvalidNumber(ParseNumberError),
}

pub enum ParseNumberError {
    InvalidFloat(ParseFloatError),
    InvalidInt(ParseIntError),
}

fn number() -> impl Parse<Output = Number> {
    (parsely::int::<i64>().map(|n| Number(N::Int(n))))
        .or(parsely::float::<f64>().map(|n| Number(N::Float(n))))
}

fn bool() -> impl Parse<Output = bool> {
    parsely::token("true")
        .map(|_| true)
        .or(parsely::token("false").map(|_| false))
}

fn null() -> impl Parse<Output = Value> {
    parsely::token("null").map(|_| Value::Null)
}

fn string() -> impl Parse<Output = String> {
    parsely::combinator::pad(
        char('"'),
        char('"'),
        escape()
            // this is particularly dense - I got stuck writing this because I forgot that char_if returned &str instead of char!
            // also mapping &str -> char to create a "parser" that outputs chars that we then further map to collect into a string...?
            // there's got to be a simpler way of matching strings?
            .or(parsely::char_if(|c| c != '"' && c != '\\').map(|s| s.chars().next().unwrap()))
            .many(..)
            .map(|chars| chars.into_iter().collect::<String>()),
    )
}

fn escape() -> impl Parse<Output = char> {
    char('\\').skip_then(
        (char('\\').map(|_| '\\')) //
            .or(char('t').map(|_| '\t'))
            .or(char('n').map(|_| '\n'))
            .or(char('r').map(|_| '\r'))
            .or(char('"').map(|_| '"')),
    )
}

fn array(input: &str) -> ParseResult<'_, Vec<Value>> {
    parsely::combinator::pad(
        char('['),
        char(']'),
        value().many(..).delimiter(char(',').then(ws().many(..))),
    )
    .parse(input)
}

fn map(input: &str) -> ParseResult<'_, Map<String, Value>> {
    parsely::combinator::pad(
        char('{').then(ws().many(..)),
        ws().many(..).then(char('}')),
        string().then_skip(char(':').pad()).then(value()).optional(),
    )
    .map(|inner| {
        let mut map = BTreeMap::new();
        if let Some((k, v)) = inner {
            map.insert(k, v);
        }
        Map(map)
    })
    .parse(input)
}

// placeholder that only matches `4` for now
fn value() -> impl Parse<Output = Value> {
    null()
        .or(bool().map(Value::Bool))
        .or(number().map(Value::Number))
        .or(string().map(Value::String))
        .or(array.map(Value::Array))
        .or(map.map(Value::Object))
        .pad()
}

fn json(input: &str) -> ParseResult<'_, Value> {
    value().pad().parse(input)
}

fn main() -> Result<(), parsely::Error> {
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
    fn arrays() -> Result<(), parsely::Error> {
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
    fn primitives() -> Result<(), parsely::Error> {
        assert_eq!(json("1")?.0, Value::Number(Number(N::Int(1))));
        assert_eq!(
            json(r#""string""#)?.0,
            Value::String(String::from("string"))
        );
        assert_eq!(json(r"true")?.0, Value::Bool(true));
        assert_eq!(json(r"false")?.0, Value::Bool(false));
        assert_eq!(json(r"null")?.0, Value::Null);

        Ok(())
    }

    #[test]
    fn maps() -> Result<(), parsely::Error> {
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

        Ok(())
    }

    #[test]
    fn whitespace() -> Result<(), parsely::Error> {
        assert_eq!(
            format!("{}", json("{ \"foo\" \t \n :  \"bar\"\t}")?.0),
            r#"{"foo": "bar",}"#,
            "spaces should be ignored"
        );

        assert_eq!(
            format!("{}", json("{ \"foo\" \t \n :  \"bar\"\t}")?.0),
            r#"{"foo": "bar",}"#,
            "other whitepsace should also be ignored"
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
}
