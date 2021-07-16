use crate::*;
use druid::widget::{Flex, Label, TextBox, Maybe};
use druid::{Data, Lens, LensExt, Widget, WidgetExt};

#[derive(Debug, Clone, Data, Lens)]
pub struct CString {
    value: Option<String>,
    #[data(ignore)]
    name: Option<String>,
}

impl CString {
    fn new() -> Self {
        Self { value: None, name: None }
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

    pub fn state(&self) -> State {
        match &self.value {
            Some(_) => State::Valid,
            None => State::None,
        }
    }

    pub fn widget() -> impl Widget<Self> {
        Flex::row()
            .with_child(Maybe::or_empty(|| Label::dynamic(|data: &String, _| data.clone() + ":")).lens(Self::name))
            .with_child(TextBox::new().lens(Self::value.map(
                |value| match value {
                    Some(v) => v.to_owned(),
                    None => "".to_owned(),
                },
                |value: &mut Option<String>, x| *value = Some(x),
            )))
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

    pub fn gui_name(mut self, name: String) -> Self {
        self.inner.name = Some(name);
        self
    }

    pub fn build(self) -> CString {
        self.inner
    }
}
