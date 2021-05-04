use thiserror::Error;

#[derive(Error, Debug)]
pub enum ConfigError {
    #[error(transparent)]
    Required(#[from] RequiredError),

    #[error("Loading Error")]
    Load(#[from] serde_yaml::Error),

    #[error(transparent)]
    Invalid(#[from] InvalidError),
}

#[derive(Error, Debug)]
#[error("{msg:?}")]
pub struct InvalidError {
    msg: String,
}

impl InvalidError {
    pub fn new<T>(str: T) -> Self
    where
        String: From<T>,
    {
        Self { msg: str.into() }
    }
}

#[derive(Error, Debug)]
#[error("Field {field:?} is required. Msg: {msg:?}")]
pub struct RequiredError {
    field: String,
    msg: String,
}

impl RequiredError {
    pub fn new(field: &str, msg: &str) -> Self {
        Self {
            field: field.to_owned(),
            msg: msg.to_owned(),
        }
    }
}
