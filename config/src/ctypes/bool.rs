use crate::*;

#[derive(Debug, Clone)]
pub struct CBool {
    value: Option<bool>,
}

impl CBool {
    fn new() -> Self {
        Self { value: None }
    }
    pub fn get(&self) -> Option<bool> {
        self.value
    }

    pub fn set(&mut self, value: bool) {
        self.value = Some(value);
    }
    pub fn unset(&mut self) {
        self.value = None
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
        self.inner.set(value);
        self
    }
    pub fn build(self) -> CBool {
        self.inner
    }
}
