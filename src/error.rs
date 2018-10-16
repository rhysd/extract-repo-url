use regex;
use std::{fmt, io};

pub type Result<T> = ::std::result::Result<T, Error>;

#[derive(PartialEq)]
pub enum Error {
    EmptyText,
    IoFailure(String),
    RepoUrlNotFound(String),
    ClipboardReadFailure(String),
    TryNextHost,
    InvalidRegex(regex::Error),
    OpenNotSupported,
    CannotOpenUrl(String),
}

impl fmt::Debug for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let msg = match self {
            Error::EmptyText => "Input text is empty".to_string(),
            Error::IoFailure(msg) => format!("IO failure: {}", msg),
            Error::RepoUrlNotFound(text) => format!("No repository URL was found in '{}'", text),
            Error::TryNextHost => unreachable!(),
            Error::ClipboardReadFailure(msg) => {
                format!("Could not read clipboard content: {}", msg)
            }
            Error::InvalidRegex(inner) => format!("{}", inner),
            Error::OpenNotSupported => "Cannot open a browser on this OS".to_string(),
            Error::CannotOpenUrl(u) => format!("Cannot open URL '{}'", u),
        };
        write!(f, "{}", msg)
    }
}

impl From<io::Error> for Error {
    fn from(inner: io::Error) -> Error {
        Error::IoFailure(format!("{}", inner))
    }
}

impl From<Box<::std::error::Error>> for Error {
    fn from(inner: Box<::std::error::Error>) -> Error {
        Error::ClipboardReadFailure(format!("{:?}", inner))
    }
}

impl From<regex::Error> for Error {
    fn from(inner: regex::Error) -> Error {
        Error::InvalidRegex(inner)
    }
}
