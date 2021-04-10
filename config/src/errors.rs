use std::error::Error;
use std::fmt;

#[derive(Debug, Clone)]
pub struct ValueRequiredError {
    path: String,
}

impl ValueRequiredError {
    pub fn new(path: String) -> Self {
        Self { path }
    }

    pub fn add(mut self, segment: &str) -> Self {
        self.path = format! {"{}::{}", segment, self.path};
        self
    }
}

impl fmt::Display for ValueRequiredError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Value {} was not set, but required", self.path)
    }
}

impl Error for ValueRequiredError {}

#[derive(Debug, Clone)]
pub struct ValueError {
    msg: String,
}

impl ValueError {
    pub fn new(msg: String) -> Self {
        Self { msg }
    }

    pub fn get_msg(&self) -> &String {
        &self.msg
    }
}

impl fmt::Display for ValueError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "ValueError with msg: {}", self.msg)
    }
}

impl Error for ValueError {}
