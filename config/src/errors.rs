use std::backtrace::Backtrace;

use thiserror::Error;

#[derive(Error, Debug)]
pub enum ConfigError {
    #[error(transparent)]
    Required(#[from] RequiredError),

    #[error(transparent)]
    Invalid(#[from] InvalidError),
}

#[derive(Error, Debug)]
#[error("{msg:?}")]
pub struct InvalidError {
    pub msg: &'static str,
}

impl InvalidError {
    pub fn new<T>(str: T) -> Self
    where
        &'static str: From<T>,
    {
        Self { msg: str.into() }
    }

    pub fn into_msg(self) -> String {
        self.msg.to_owned()
    }
}

#[derive(Error, Debug)]
#[error("got error at field {field:?} is required. Msg: {msg:?}")]
pub struct RequiredError {
    pub field: &'static str,
    pub msg: &'static str,
    backtrace: Backtrace,
}

impl RequiredError {
    pub fn new(field: &'static str, msg: &'static str) -> Self {
        Self {
            field,
            msg,
            backtrace: Backtrace::capture(),
        }
    }
}
