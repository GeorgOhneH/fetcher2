use std::backtrace::Backtrace;

use thiserror::Error;
use tokio::time::error::Elapsed;

pub type Result<T> = std::result::Result<T, TError>;

#[derive(Error, Debug)]
#[error("{kind:?}")]
pub struct TError {
    pub kind: TErrorKind,
    pub backtrace: Backtrace,
}

impl TError {
    pub fn new(kind: TErrorKind) -> Self {
        Self {
            kind,
            backtrace: Backtrace::capture(),
        }
    }
}

impl<T> From<T> for TError
where
    TErrorKind: From<T>,
{
    fn from(err: T) -> Self {
        Self::new(err.into())
    }
}

#[derive(Error, Debug)]
pub enum TErrorKind {
    #[error("Previous login attempt was unsuccessful")]
    PreviousLoginError,

    #[error("For this module is the username/password required")]
    LoginDataRequired,

    #[error("Login Data was not correct")]
    LoginError,

    #[error("Got unexpected data from server")]
    WrongFormat,

    #[error("The Etag was not well formatted")]
    ETagFormat,

    #[error("Xml error: {0}")]
    Xml(String),

    #[error("Url Parse Error")]
    UrlParseError(#[from] url::ParseError),

    #[error("Client Error")]
    ClientError(#[from] reqwest::Error),

    #[error("Timeout error")]
    TimeOut(#[from] Elapsed),

    #[error("File Error")]
    FileError(#[from] std::io::Error),

    #[error("Serde Error")]
    SerdeError(#[from] ron::Error),

    #[error("Config Error")]
    ConfigError(#[from] config::ConfigError),
}

pub trait TErrorFast<T> {
    fn wrong_format(self) -> Result<T>;
}

impl<T> TErrorFast<T> for Option<T> {
    fn wrong_format(self) -> Result<T> {
        self.ok_or_else(|| TError::new(TErrorKind::WrongFormat))
    }
}
