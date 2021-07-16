use crate::task::Task;
use std::backtrace::Backtrace;
use thiserror::Error;
use tokio::time::error::Elapsed;
use std::ops::FromResidual;
use std::convert::Infallible;
use druid::ExtEventError;

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

    #[error("Login Data was not correct")]
    LoginError,

    #[error("Expected different format from data")]
    WrongFormat,

    #[error("The Etag was not well formatted")]
    ETagFormat,

    #[error("Something")]
    Something,

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
    SerdeError(#[from] serde_yaml::Error),

    #[error("Utf8 Error")]
    Utf8Error(#[from] std::string::FromUtf8Error),

    #[error("Config Error")]
    ConfigError(#[from] config::ConfigError),

    #[error("Join Error")]
    JoinError(#[from] tokio::task::JoinError),

    #[error("Send Error")]
    SendError(#[from] tokio::sync::mpsc::error::SendError<Task>),

    #[error("Druid ExtEvent Error")]
    ExtEventError(#[from] ExtEventError),
}

pub trait DefaultOk<T> {
    fn d_ok(self) -> Result<T>;
}

impl<T> DefaultOk<T> for Option<T> {
    fn d_ok(self) -> Result<T> {
        self.ok_or_else(|| TError::new(TErrorKind::Something))
    }
}
