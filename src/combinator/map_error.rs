//! Combinators that add custom error reporting when their contained parser/lexer fails

use crate::Error;

/// Map parsely errors into a custom error type.
///
/// MapError can be used to inject custom errors duroing parsing
pub struct MapError<T, E: std::error::Error> {
    inner: T,
    error: Box<dyn Fn(Error) -> E>,
}
