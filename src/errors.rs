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
}
