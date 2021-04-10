use crate::*;

#[derive(Debug, Clone)]
pub struct ConfigArgString {
    value: Option<String>,
}

impl ConfigArgString {
    fn new() -> Self {
        Self { value: None }
    }

    pub fn get(&self) -> Option<&String> {
        Option::from(&self.value)
    }

    pub fn set(&mut self, value: Option<String>) {
        self.value = value;
    }
}

pub struct ConfigArgStringBuilder {
    inner: ConfigArgString,
}

impl ConfigArgStringBuilder {
    pub fn new() -> Self {
        Self {
            inner: ConfigArgString::new(),
        }
    }
    pub fn default(mut self, value: String) -> Self {
        self.inner.set(Some(value));
        self
    }
    pub fn build(self) -> ConfigArgString {
        self.inner
    }
}
