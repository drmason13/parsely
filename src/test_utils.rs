#![doc(hidden)]
#![cfg(test)]
use crate::{ParseOutput, Parser};

pub(crate) fn test_parser(
    test_index: usize,
    test_name: &str,
    parser: &mut impl Parser,
    input: &str,
    expected_output: Option<&str>,
    expected_remaining: &str,
) {
    assert_eq!(
        parser.parse(input),
        ParseOutput::new(expected_output, expected_remaining),
        "{} parser - {test_name}:{test_index}",
        parser.name(),
    );
}

pub(crate) fn test_parser_batch(
    test_name: &str,
    mut parser: impl Parser,
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
