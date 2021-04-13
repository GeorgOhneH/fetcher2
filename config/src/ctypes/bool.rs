use crate::*;

#[derive(Debug, Clone)]
pub struct CBool {
    value: Option<bool>,
}

impl CBool {
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

pub struct CBoolBuilder {
    inner: CBool,
}

impl CBoolBuilder {
    pub fn new() -> Self {
        Self {
            inner: CBool::new(),
        }
    }
    pub fn default(mut self, value: bool) -> Self {
        self.inner.set(Some(value));
        self
    }
    pub fn build(self) -> CBool {
        self.inner
    }
}
