use crate::*;
use druid::{Data, Lens, Widget, WidgetExt, LensExt};
use druid::widget::TextBox;

#[derive(Debug, Clone, Data, Lens)]
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

    pub fn set(&mut self, value: String) {
        self.value = Some(value);
    }
    pub fn unset(&mut self) {
        self.value = None
    }

    pub fn widget() -> impl Widget<Self> {
        TextBox::new().lens(Self::value.map(
            |value| match value {
                Some(v) => v.to_owned(),
                None => "".to_owned(),
            },
            |value: &mut Option<String>, x| *value = Some(x),
        ))
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
        self.inner.set(value);
        self
    }
    pub fn build(self) -> CString {
        self.inner
    }
}
