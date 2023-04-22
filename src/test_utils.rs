#![doc(hidden)]
#![cfg(test)]
use std::fmt;

use crate::{Lex, Parse};

pub(crate) fn test_lexer(
    test_index: usize,
    test_name: &str,
    lexer: &mut (impl Lex + fmt::Debug),
    input: &str,
    expected_output: &str,
    expected_remaining: &str,
) {
    assert_eq!(
        (expected_output, expected_remaining),
        lexer
            .lex(input)
            .unwrap_or_else(|_| panic!("{lexer:?} lexer - {test_name}:{test_index} should match")),
        "{lexer:?} lexer - {test_name}:{test_index}. Expected left; Got right",
    );
}

pub(crate) fn test_lexer_error(
    test_index: usize,
    test_name: &str,
    lexer: &mut (impl Lex + fmt::Debug),
    input: &str,
) {
    assert!(
        lexer.lex(input).is_err(),
        "{lexer:?} lexer - {test_name}:{test_index} should error"
    );
}

pub(crate) fn test_lexer_batch(
    test_name: &str,
    mut lexer: impl Lex + fmt::Debug,
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
            )
        } else {
            test_lexer_error(i, test_name, &mut lexer, input)
        }
    }
}

pub(crate) fn test_parser<T: PartialEq + fmt::Debug>(
    test_index: usize,
    test_name: &str,
    parser: &mut (impl Parse<Output = T> + fmt::Debug),
    input: &str,
    expected_output: T,
    expected_remaining: &str,
) {
    assert_eq!(
        (expected_output, expected_remaining),
        parser.parse(input).unwrap_or_else(|_| panic!(
            "{parser:?} parser - {test_name}:{test_index} should match"
        )),
        "{parser:?} parser - {test_name}:{test_index}. Expected left; Got right",
    );
}

pub(crate) fn test_parser_error<T>(
    test_index: usize,
    test_name: &str,
    parser: &mut (impl Parse<Output = T> + fmt::Debug),
    input: &str,
) {
    assert!(
        parser.parse(input).is_err(),
        "{parser:?} parser - {test_name}:{test_index} should error"
    );
}

pub(crate) fn test_parser_batch<T: PartialEq + Clone + fmt::Debug>(
    test_name: &str,
    mut parser: impl Parse<Output = T> + fmt::Debug,
    cases: &[(&str, Option<T>, &str)],
) {
    for (i, (input, expected_output, expected_remaining)) in cases.iter().enumerate() {
        if let Some(expected_output) = expected_output {
            test_parser(
                i,
                test_name,
                &mut parser,
                input,
                expected_output.clone(),
                expected_remaining,
            )
        } else {
            test_parser_error(i, test_name, &mut parser, input)
        }
    }
}
