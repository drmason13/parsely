#![doc(hidden)]
#![cfg(test)]
use std::fmt::Display;

use crate::{Lex, Parse};

pub(crate) fn test_lexer(
    test_index: usize,
    test_name: &str,
    lexer: &mut (impl Lex + Display),
    input: &str,
    expected_output: &str,
    expected_remaining: &str,
    should_match: bool,
) {
    if should_match {
        assert_eq!(
            (expected_output, expected_remaining),
            lexer.lex(input).unwrap_or_else(|_| panic!(
                "{lexer} lexer - {test_name}:{test_index} should match"
            )),
            "{lexer} lexer - {test_name}:{test_index}. Expected left; Got right",
        );
    } else {
        assert!(lexer.lex(input).is_err(),);
    }
}

pub(crate) fn test_lexer_batch(
    test_name: &str,
    mut lexer: impl Lex + Display,
    cases: &[(&str, Option<&str>, &str)],
) {
    for (i, (input, expected_output, expected_remaining)) in cases.iter().enumerate() {
        if let Some(expected_output) = expected_output {
            test_lexer(
                i,
                test_name,
                &mut lexer,
                input,
                expected_output,
                expected_remaining,
                true,
            )
        } else {
            test_lexer(
                i,
                test_name,
                &mut lexer,
                input,
                "",
                expected_remaining,
                false,
            )
        }
    }
}

pub(crate) fn test_parser<T>(
    test_index: usize,
    test_name: &str,
    parser: &mut (impl Lex + Display),
    input: &str,
    expected_output: T,
    expected_remaining: &str,
    should_match: bool,
) {
    if should_match {
        assert_eq!(
            (expected_output, expected_remaining),
            parser.parse(input).unwrap_or_else(|_| panic!(
                "{parser} parser - {test_name}:{test_index} should match"
            )),
            "{parser} parser - {test_name}:{test_index}. Expected left; Got right",
        );
    } else {
        assert!(parser.parse(input).is_err(),);
    }
}

pub(crate) fn test_parser_batch<T>(
    test_name: &str,
    mut parser: impl Parse<Output = T> + Display,
    cases: &[(&str, Option<T>, &str)],
) {
    for (i, (input, expected_output, expected_remaining)) in cases.iter().enumerate() {
        if let Some(expected_output) = expected_output {
            test_parser(
                i,
                test_name,
                &mut parser,
                input,
                expected_output,
                expected_remaining,
                true,
            )
        } else {
            test_parser(
                i,
                test_name,
                &mut parser,
                input,
                "",
                expected_remaining,
                false,
            )
        }
    }
}
