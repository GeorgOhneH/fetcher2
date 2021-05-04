use crate::{CType, ConfigError};
use serde_yaml::Value;

#[derive(Debug, Clone)]
pub struct CWrapper {
    inner: CType,
    kind: CWrapperKind,
}

impl CWrapper {
    fn new(inner: CType, kind: CWrapperKind) -> Self {
        Self { inner, kind }
    }

    pub fn inner(&self) -> &CType {
        &self.inner
    }

    pub fn inner_mut(&mut self) -> &mut CType {
        &mut self.inner
    }

    pub fn into_inner(self) -> CType {
        self.inner
    }

    pub fn kind(&self) -> &CWrapperKind {
        &self.kind
    }

    pub(crate) fn consume_value(&mut self, value: Value) -> Result<(), ConfigError> {
        self.inner.consume_value(value)
    }
}

#[derive(Debug, Clone)]
pub enum CWrapperKind {
    Mutex,
    RwLock,
    Arc,
}

pub struct CWrapperBuilder {
    inner: CWrapper,
}

impl CWrapperBuilder {
    pub fn new(inner: CType, kind: CWrapperKind) -> CWrapperBuilder {
        Self {
            inner: CWrapper::new(inner, kind),
        }
    }

    pub fn build(self) -> CWrapper {
        self.inner
    }
}
