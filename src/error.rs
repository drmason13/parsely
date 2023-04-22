use std::fmt;

#[non_exhaustive]
#[derive(PartialEq, Debug)]
pub enum Error {
    NoMatch,
    FailedConversion,
}

impl std::error::Error for Error {}
impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::NoMatch => write!(f, "No Match"),
            Error::FailedConversion => write!(f, "Failed to convert matched input"),
        }
    }
}
