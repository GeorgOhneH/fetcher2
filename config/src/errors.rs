use std::error::Error;
use std::fmt;
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RequiredError {
    path: String,
}

impl RequiredError {
    pub fn new(path: String) -> Self {
        Self { path }
    }

    pub fn add(mut self, segment: &str) -> Self {
        self.path = format! {"{}::{}", segment, self.path};
        self
    }
}

impl fmt::Display for RequiredError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Value {} was not set, but required", self.path)
    }
}

impl Error for RequiredError {}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MsgError {
    msg: String,
}

impl MsgError {
    pub fn new(msg: String) -> Self {
        Self { msg }
    }

    pub fn get_msg(&self) -> &String {
        &self.msg
    }
}

impl fmt::Display for MsgError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "ValueError with msg: {}", self.msg)
    }
}
