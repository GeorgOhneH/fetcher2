use crate::task::Task;
use std::backtrace::Backtrace;
use std::option::NoneError;
use thiserror::Error;

pub type Result<T> = std::result::Result<T, TError>;

#[derive(Error, Debug)]
#[error("{kind:?}")]
pub struct TError {
    kind: TErrorKind,
    backtrace: Backtrace,
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

    #[error("Expected different format from data")]
    WrongFormat,

    #[error("The Etag was not well formatted")]
    ETagFormat,

    #[error("Url Parse Error")]
    UrlParseError(#[from] url::ParseError),

    #[error("Client Error")]
    ClientError(#[from] reqwest::Error),

    #[error("File Error")]
    FileError(#[from] async_std::io::Error),

    #[error("Serde Error")]
    SerdeError(#[from] serde_yaml::Error),

    #[error("Utf8 Error")]
    Utf8Error(#[from] std::string::FromUtf8Error),

    #[error("Config Error")]
    ConfigError(#[from] config::ConfigError),

    #[error("Join Error")]
    JoinError(#[from] tokio::task::JoinError),

    #[error("Send Error")]
    SendError(#[from] tokio::sync::mpsc::error::SendError<Task>),
}

impl From<NoneError> for TErrorKind {
    fn from(_: NoneError) -> Self {
        TErrorKind::WrongFormat
    }
}
