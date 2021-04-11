use crate::*;

#[derive(Debug, Clone)]
pub struct ConfigArgBool {
    value: Option<bool>,
}

impl ConfigArgBool {
    fn new() -> Self {
        Self { value: None }
    }
    pub fn get(&self) -> Option<&bool> {
        Option::from(&self.value)
    }

    pub fn set(&mut self, value: Option<bool>) {
        self.value = value;
    }
}

pub struct ConfigArgBoolBuilder {
    inner: ConfigArgBool,
}

impl ConfigArgBoolBuilder {
    pub fn new() -> Self {
        Self {
            inner: ConfigArgBool::new(),
        }
    }
    pub fn default(mut self, value: bool) -> Self {
        self.inner.set(Some(value));
        self
    }
    pub fn build(self) -> ConfigArgBool {
        self.inner
    }
}
