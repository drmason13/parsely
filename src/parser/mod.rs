//! The built in parsers provided by parsely
//!
mod number;
mod switch;

pub use self::number::{float, int, number, uint};
pub use self::switch::switch;
