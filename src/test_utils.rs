#![doc(hidden)]
#![cfg(test)]
use std::fmt::Display;

use crate::{ParseResult, Parser};

pub(crate) fn test_parser(
    test_index: usize,
    test_name: &str,
    parser: &mut (impl Parser + Display),
    input: &str,
    expected_output: Option<&str>,
    expected_remaining: &str,
) {
    assert_eq!(
        ParseResult::new(expected_output, expected_remaining),
        parser.parse(input),
        "{parser} parser - {test_name}:{test_index}. Expected left; Got right",
    );
}

pub(crate) fn test_parser_batch(
    test_name: &str,
    mut parser: impl Parser + Display,
    cases: &[(&str, Option<&str>, &str)],
) {
    for (i, (input, expected_output, expected_remaining)) in cases.iter().enumerate() {
        test_parser(
            i,
            test_name,
            &mut parser,
            input,
            *expected_output,
            expected_remaining,
        )
    }
}
