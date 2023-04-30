//! [`std::str::pattern`] is still an unstable API (as of rust 1.96.0)
//!
//! This module serves to wrap this unstable API and expose it in a stable way
//! similar to how [`str::find()`] is a stable function despite using the unstable Pattern API in its implementation.
//!
//! Unfortunately this is done in a fairly crude fasion currently, using an enum over *some of* the types implementing [`Pattern`](std::str::pattern::Pattern).
//!
//! Notably, `FnMut(char) -> bool` and [char; N] are missing from this enum.

#[derive(Clone)]
pub enum PatternLite<'a> {
    Str(&'a str),
    Char(char),
    CharSlice(&'a [char]),
}

impl<'a> From<&'a str> for PatternLite<'a> {
    fn from(value: &'a str) -> Self {
        PatternLite::Str(value)
    }
}

impl<'a> From<char> for PatternLite<'a> {
    fn from(value: char) -> Self {
        PatternLite::Char(value)
    }
}

impl<'a> From<&'a [char]> for PatternLite<'a> {
    fn from(value: &'a [char]) -> Self {
        PatternLite::CharSlice(value)
    }
}
