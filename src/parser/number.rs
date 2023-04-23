use std::str::FromStr;

use crate::{char, digit, Lex, Parse};

/// Parses a signed integer, i.e. one or more base 10 digits with or without a leading '-' indicating the sign.
///
/// To parse unsigned integers that forbid the leading '-' consider using:
/// * [`uint()`] which will parse only base 10 digits
/// * [`digit().many(1..)`] which is the lexer that powers [`uint()`]
///
/// To parse decimals consider using:
/// * [`float()`] which will parse only decimals
/// * [`number()`] which will parse integers or decimals
///
pub fn int<T: FromStr>() -> impl Parse<Output = T> {
    char('-')
        .optional()
        .then(digit().many(1..))
        .try_map(FromStr::from_str)
}

pub fn uint<T: FromStr>() -> impl Parse<Output = T> {
    digit().many(1..).try_map(FromStr::from_str)
}

pub fn float<T: FromStr>() -> impl Parse<Output = T> {
    char('-')
        .optional()
        .then(digit().many(1..))
        // not every language uses '.' for decimals, but rust float parsing expects it
        .then(char('.').or(char(',')))
        .then(digit().many(0..))
        .try_map(|s| {
            let s = s.replace(',', ".");
            FromStr::from_str(&s)
        })
}

pub fn number<T: FromStr>() -> impl Parse<Output = T> {
    float::<T>().or(int::<T>())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_utils::*;

    #[test]
    fn rust_float_parse() {
        assert_eq!("34.0".parse::<f32>().unwrap(), 34.0);
        assert_eq!("34.".parse::<f32>().unwrap(), 34.0);
    }

    #[test]
    fn parsing() {
        test_parser_batch(
            "int matches base 10 digits",
            int::<u8>(),
            &[
                ("abc", None, "abc"), //
                ("123", Some(123), ""),
                ("1.23", Some(1), ".23"),
            ],
        );

        test_parser_batch(
            "float matches only decimals",
            float::<f32>(),
            &[
                ("12.6", Some(12.6), ""),
                ("12.", Some(12.), ""),
                ("123", None, "123"),
                ("12.3A", Some(12.3), "A"),
                ("12.A3", Some(12.), "A3"),
                ("12.0.1", Some(12.0), ".1"),
            ],
        );

        test_parser_batch(
            "float matches decimals with ,",
            float::<f32>(),
            &[
                ("12,6", Some(12.6), ""),
                ("12,", Some(12.), ""),
                ("123", None, "123"),
                ("12,3A", Some(12.3), "A"),
                ("12,A3", Some(12.), "A3"),
                ("12,0.1", Some(12.0), ".1"),
            ],
        );

        test_parser_batch(
            "number matches base 10 digits or decimals",
            number::<f32>(),
            &[
                ("12.6", Some(12.6), ""),
                ("12.", Some(12.), ""),
                ("123", Some(123.), ""),
                ("12.3A", Some(12.3), "A"),
                ("12.A3", Some(12.), "A3"),
                ("12,0.1", Some(12.0), ".1"),
            ],
        );
    }
}
