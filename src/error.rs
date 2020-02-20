use std::error::Error as ErrorTrait;
use std::fmt;

#[derive(Debug, PartialEq)]
pub enum Error {
    IncompleteOctet,
    InvalidCharacter(char),
}

impl ErrorTrait for Error {
    fn source(&self) -> Option<&(dyn ErrorTrait + 'static)> {
        None
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::IncompleteOctet => write!(f, "Octet was not complete"),
            Error::InvalidCharacter(c) => write!(f, "'{}' is not valid base16", c),
        }
    }
}
