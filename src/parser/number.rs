use std::str::FromStr;

use crate::{char, char_if, digit, non_zero_digit, Lex, Parse};

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
        .then(char_if(|c| c.is_ascii_digit() && c != '0'))
        .then(digit().many(0..100_000))
        .try_map(FromStr::from_str)
}

pub fn uint<T: FromStr>() -> impl Parse<Output = T> {
    non_zero_digit()
        .then(digit().many(0..100_000))
        .try_map(FromStr::from_str)
}

/// Parses a floating point decimal in standard notation (not scientific notation)
///
/// # Examples
///
/// Basic usage:
///
/// ```
/// use parsely::{float, Parse};
///
/// let (output, remaining) = float::<f32>().parse("123.456")?;
/// assert_eq!(output, 123.456);
/// assert_eq!(remaining, "");
///
///
/// // Warning! This isn't interpreted correctly by float()
/// let (output, remaining) = float::<f32>().parse("6.78e-9")?;
/// assert_eq!(output, 6.78);
/// # Ok::<(), parsely::Error>(())
/// ```
///
/// Commas are accepted:
///
/// ```
/// # use parsely::{number, Parse};
/// let (output, remaining) = number::<f32>().parse("123,456")?;
/// assert_eq!(output, 123.456);
/// assert_eq!(remaining, "");
/// # Ok::<(), parsely::Error>(())
/// ```
pub fn float<T: FromStr>() -> impl Parse<Output = T> {
    char('-')
        .optional()
        .then(non_zero_digit())
        .then(digit().many(0..100_000))
        .then(char('.').or(char(',')))
        .then(digit().many(0..100_000))
        // not every language uses '.' for decimals, but rust float parsing expects it
        .try_map(|s| {
            let s = s.replace(',', ".");
            FromStr::from_str(&s)
        })
}

/// Parses a float or an int.
///
/// # Examples
///
/// Basic usage:
///
/// ```
/// use parsely::{number, Parse};
///
/// let (output, remaining) = number::<f32>().parse("123.456")?;
/// assert_eq!(output, 123.456);
/// assert_eq!(remaining, "");
///
/// // you can match an integer and return a float representing it
/// let (output, remaining) = number::<f64>().parse("123")?;
/// assert_eq!(output, 123.0);
/// assert_eq!(remaining, "");
/// # Ok::<(), parsely::Error>(())
/// ```
///
/// Use f32 or f64 if you want to parse and store either floats or integers
///
/// ```
/// # use parsely::{number, Parse};
/// let (output, remaining) = number::<u8>().parse("123.456")?;
///
/// // only the integer is matched because the float failed to convert to a u8
/// assert_eq!(output, 123);
/// assert_eq!(remaining, ".456");
/// # Ok::<(), parsely::Error>(())
/// ```
///
/// This happens because
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
