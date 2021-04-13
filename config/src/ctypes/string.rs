use crate::*;

#[derive(Debug, Clone)]
pub struct CString {
    value: Option<String>,
}

impl CString {
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

pub struct CStringBuilder {
    inner: CString,
}

impl CStringBuilder {
    pub fn new() -> Self {
        Self {
            inner: CString::new(),
        }
    }
    pub fn default(mut self, value: String) -> Self {
        self.inner.set(Some(value));
        self
    }
    pub fn build(self) -> CString {
        self.inner
    }
}
