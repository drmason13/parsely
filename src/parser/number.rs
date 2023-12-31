//! This module provides built-in Parsers for numbers such as float, int, uint and number.
//!
//!
//! # Specifying output type
//!
//! These parsers require a generic parameter to indicate the exact type of number they should output, which will usually require a turbofish: `uint::<u8>()`.
//!
//! Note:
//!
//! There is unfortunately no compile time protection against specifying somewhat non-sensical types such as:
//! * `float::<u8>()` - which will never be able to fully match a decimal input like `123.45` because it will match `123` as a u8 instead.
//! * `int::<Ipv4Addr>()` - which will never be able to parse anything because int does not expect a `.` in the input.
//!
//! We don't consider this a common enough problem to use any complex numerical traits to bound the types to avoid this.
//!
//! # Maximum number of digits
//!
//! These parsers parse a maximum of 100_000 digits (plus 100_000 decimal places in the case of [`float`]), which is probably plenty right?
//!
//! This number isn't stable though, so try not to depend on the fact somehow!
//!
//! I decided to avoid an unbound number of digits so it was more robust in the face of malicious input, but this library has not been tested for security yet.

use std::str::FromStr;

use crate::{char, char_if, digit, non_zero_digit, Lex, Parse};

/// Parses a signed integer, i.e. one or more base 10 digits with or without a leading '-' indicating the sign.
///
/// To parse unsigned integers that forbid the leading '-' consider using:
/// * [`uint()`] which will parse only base 10 digits
///
/// To parse decimals consider using:
/// * [`float()`] which will parse only decimals
/// * [`number()`] which will parse integers or decimals
///
/// # Examples
///
/// Basic usage:
///
/// ```
/// use parsely::{int, Parse};
///
/// assert_eq!(int().parse("123")?, (123, ""));
/// # Ok::<(), parsely::InProgressError>(())
/// ```
///
pub fn int<T: FromStr + Clone>() -> impl Parse<Output = T> + Clone {
    char('-')
        .optional()
        .then(char_if(|c| c.is_ascii_digit() && c != '0'))
        .then(digit().many(0..=100_000))
        .try_map(FromStr::from_str)
}

/// Parses an unsigned integer, i.e. one or more base 10 digits.
///
/// To parse signed integers that allow a leading '-' consider using:
/// * [`int()`] which will parse only base 10 digits
///
/// To parse decimals consider using:
/// * [`float()`] which will parse only decimals
/// * [`number()`] which will parse integers or decimals
///
pub fn uint<T: FromStr + Clone>() -> impl Parse<Output = T> + Clone {
    non_zero_digit()
        .then(digit().many(0..100_000))
        .or("0")
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
/// // Scientific notation matches too
/// let (output, remaining) = float::<f32>().parse("6.78e-9")?;
/// assert_eq!(output, 6.78e-9);
/// # Ok::<(), parsely::InProgressError>(())
/// ```
///
/// Commas are not accepted:
///
/// ```
/// # use parsely::{number, Parse};
/// let (output, remaining) = number::<f32>().parse("123,456")?;
/// assert_eq!(output, 123.0);
/// assert_eq!(remaining, ",456");
/// # Ok::<(), parsely::InProgressError>(())
/// ```
pub fn float<T: FromStr>() -> impl Parse<Output = T> {
    float_scientific_notation().or('-'
        .optional()
        .then(non_zero_digit())
        .then(digit().many(0..100_000))
        .then('.')
        .then(digit().many(0..100_000))
        .try_map(FromStr::from_str))
}

pub fn float_scientific_notation<T: FromStr>() -> impl Parse<Output = T> {
    ('-'.optional())
        .then(non_zero_digit())
        .then(digit().many(0..100_000))
        .then('.')
        .then(digit().many(0..100_000))
        .then('e'.or('E'))
        .then('-'.or('+').optional())
        .then(digit().many(0..100_000))
        .try_map(FromStr::from_str)
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
/// # Ok::<(), parsely::InProgressError>(())
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
/// # Ok::<(), parsely::InProgressError>(())
/// ```
///
/// This happens because
pub fn number<T: FromStr + Clone>() -> impl Parse<Output = T> {
    float::<T>().or(int::<T>())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_utils::*;

    #[test]
    fn rust_float_parse() {
        assert_eq!("1.0e1".parse::<f32>().unwrap(), 10.0);
        assert_eq!("1.0e+1".parse::<f32>().unwrap(), 10.0);
        assert_eq!("1.0e-1".parse::<f32>().unwrap(), 0.1);
        assert_eq!("-1.0e1".parse::<f32>().unwrap(), -10.0);
        assert_eq!("-1.0e+1".parse::<f32>().unwrap(), -10.0);
        assert_eq!("-1.0e-1".parse::<f32>().unwrap(), -0.1);
        assert_eq!("-1.0e-0".parse::<f32>().unwrap(), -1.0);
        assert_eq!("-1.0e+0".parse::<f32>().unwrap(), -1.0);

        assert_eq!("1.0E1".parse::<f32>().unwrap(), 10.0);
        assert_eq!("1.0E+1".parse::<f32>().unwrap(), 10.0);
        assert_eq!("1.0E-1".parse::<f32>().unwrap(), 0.1);
        assert_eq!("-1.0E1".parse::<f32>().unwrap(), -10.0);
        assert_eq!("-1.0E+1".parse::<f32>().unwrap(), -10.0);
        assert_eq!("-1.0E-1".parse::<f32>().unwrap(), -0.1);
        assert_eq!("-1.0E-0".parse::<f32>().unwrap(), -1.0);
        assert_eq!("-1.0E+0".parse::<f32>().unwrap(), -1.0);

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
            "float matches scientific notation",
            float::<f32>(),
            &[
                ("1.0e1", Some(10.0), ""),
                ("1.0e-1", Some(0.1), ""),
                ("2.0e-1", Some(0.2), ""),
                ("1.0e-2", Some(0.01), ""),
                ("3.0e+1", Some(30.0), ""),
                ("1.0e-0", Some(1.0), ""),
                ("3.0e+0", Some(3.0), ""),
                ("1.0E1", Some(10.0), ""),
                ("1.0E-1", Some(0.1), ""),
                ("2.0E-1", Some(0.2), ""),
                ("1.0E-2", Some(0.01), ""),
                ("3.0E+1", Some(30.0), ""),
                ("1.0E-0", Some(1.0), ""),
                ("3.0E+0", Some(3.0), ""),
            ],
        );

        test_parser_batch(
            "scientific notation works",
            float_scientific_notation::<f32>(),
            &[
                ("1.0e1", Some(10.0), ""),
                ("1.0e-1", Some(0.1), ""),
                ("2.0e-1", Some(0.2), ""),
                ("1.0e-2", Some(0.01), ""),
                ("3.0e+1", Some(30.0), ""),
                ("1.0e-0", Some(1.0), ""),
                ("3.0e+0", Some(3.0), ""),
                ("1.0E1", Some(10.0), ""),
                ("1.0E-1", Some(0.1), ""),
                ("2.0E-1", Some(0.2), ""),
                ("1.0E-2", Some(0.01), ""),
                ("3.0E+1", Some(30.0), ""),
                ("1.0E-0", Some(1.0), ""),
                ("3.0E+0", Some(3.0), ""),
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
                ("12.0.1", Some(12.0), ".1"),
            ],
        );
    }
}
