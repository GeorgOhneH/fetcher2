use thiserror::Error;

#[derive(Error, Debug)]
pub enum TemplateError {
    #[error("Previous login attempt was unsuccessful")]
    PreviousLoginError,

    #[error("Expected different format from data")]
    WrongFormat,

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
}
