#![doc(hidden)]
#![cfg(test)]
use std::fmt;

use crate::{Lex, Parse};

pub(crate) fn test_lexer(
    test_index: usize,
    test_name: &str,
    lexer: &mut impl Lex,
    input: &str,
    expected_output: &str,
    expected_remaining: &str,
) {
    assert_eq!(
        (expected_output, expected_remaining),
        lexer
            .lex(input)
            .unwrap_or_else(|_| panic!("lexer:{test_name}:{test_index} should match")),
        "lexer:{test_name}:{test_index}. Expected left; Got right",
    );
}

pub(crate) fn test_lexer_error(
    test_index: usize,
    test_name: &str,
    lexer: &mut impl Lex,
    input: &str,
) {
    assert!(
        lexer.lex(input).is_err(),
        "lexer:{test_name}:{test_index} should error"
    );
}

pub(crate) fn test_lexer_batch(
    test_name: &str,
    mut lexer: impl Lex,
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

pub(crate) fn test_parser<'i, T: PartialEq + fmt::Debug>(
    test_index: usize,
    test_name: &str,
    parser: &mut impl Parse<Output<'i> = T>,
    input: &'i str,
    expected_output: T,
    expected_remaining: &str,
) {
    assert_eq!(
        (expected_output, expected_remaining),
        parser.parse(input).unwrap_or_else(|e| panic!(
            "parser:{test_name}:{test_index} should match but had error: {e:?}"
        )),
        "parser:{test_name}:{test_index}. Expected left; Got right",
    );
}

pub(crate) fn test_parser_error<'i, T>(
    test_index: usize,
    test_name: &str,
    parser: &mut impl Parse<Output<'i> = T>,
    input: &'i str,
) {
    assert!(
        parser.parse(input).is_err(),
        "parser:{test_name}:{test_index} should error"
    );
}

pub(crate) fn test_parser_batch<'i, T: PartialEq + Clone + fmt::Debug>(
    test_name: &str,
    mut parser: impl Parse<Output<'i> = T>,
    cases: &[(&'i str, Option<T>, &'i str)],
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
