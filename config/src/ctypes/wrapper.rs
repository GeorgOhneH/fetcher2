use crate::{CType, ConfigError};
use serde_yaml::Value;
use druid::{Data, Lens, Widget, WidgetExt};
use druid::widget::Label;

impl Data for Box<CWrapper> {
    fn same(&self, other: &Self) -> bool {
        self.as_ref().same(other.as_ref())
    }
}

#[derive(Debug, Clone, Data, Lens)]
pub struct CWrapper {
    #[lens(name = "inner_lens")]
    inner: CType,
    #[lens(name = "kind_lens")]
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

    pub fn widget() -> impl Widget<Self> {
        Label::new("TODO")
        // CType::widget().lens(Self::inner_lens)
    }
}

#[derive(Debug, Clone, Data, PartialEq)]
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
