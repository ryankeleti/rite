/// Amalgamate all the errors.
use std::{fmt, io, path::PathBuf};

// Just used for command-line reporting, not handling.
#[derive(Debug)]
pub enum Error {
    Io(io::Error),
    Askama(askama::Error),
    Syntect(syntect::Error),
    SyntectLoad(PathBuf, syntect::LoadingError),
    ChronoParse(chrono::format::ParseError),
    ReadPostHeader(PathBuf, toml::de::Error),
    ReadConfig(PathBuf, toml::de::Error),
    MissingConfig(PathBuf),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::Io(e) => write!(f, "unexpected IO error: {}", e),
            Error::Askama(e) => write!(f, "failed to render askama template: {}", e),
            Error::Syntect(e) => write!(f, "failed to syntax highlight: {}", e),
            Error::SyntectLoad(path, e) => {
                write!(
                    f,
                    "failed to load syntect theme from {}: {}",
                    path.display(),
                    e
                )
            }
            Error::ChronoParse(e) => write!(f, "failed to parse datetime: {}", e),
            Error::ReadPostHeader(path, e) => write!(
                f,
                "failed to read post header from {}: {}",
                path.display(),
                e
            ),
            Error::ReadConfig(path, e) => write!(
                f,
                "failed to read configuration from {}: {}",
                path.display(),
                e
            ),
            Error::MissingConfig(path) => write!(f, "config file {} not found", path.display()),
        }
    }
}

impl From<io::Error> for Error {
    fn from(e: io::Error) -> Self {
        Self::Io(e)
    }
}

impl From<askama::Error> for Error {
    fn from(e: askama::Error) -> Self {
        Self::Askama(e)
    }
}

impl From<chrono::format::ParseError> for Error {
    fn from(e: chrono::format::ParseError) -> Self {
        Self::ChronoParse(e)
    }
}
