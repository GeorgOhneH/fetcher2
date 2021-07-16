use crate::*;
use druid::{Data, Lens, Widget, WidgetExt};
use serde_yaml::Mapping;

#[derive(Debug, Clone, Data, Lens)]
pub struct CCheckableStruct {
    inner: CStruct,
    checked: bool,
}

impl CCheckableStruct {
    fn new(config_struct: CStruct) -> Self {
        Self {
            inner: config_struct,
            checked: true,
        }
    }

    pub fn get_inner(&self) -> &CStruct {
        &self.inner
    }

    pub fn get_inner_mut(&mut self) -> &mut CStruct {
        &mut self.inner
    }

    pub fn set_checked(&mut self, checked: bool) {
        self.checked = checked;
    }

    pub fn is_checked(&self) -> bool {
        self.checked
    }

    pub(crate) fn consume_map(&mut self, map: Mapping) -> Result<(), ConfigError> {
        self.set_checked(true);
        self.inner.consume_map(map)
    }

    pub fn state(&self) -> State {
        if self.checked {
            State::None
        } else {
            self.inner.state()
        }
    }

    pub fn widget() -> impl Widget<Self> {
        CStruct::widget().lens(Self::inner)
    }
}

pub struct CCheckableStructBuilder {
    checked: Option<bool>,
    struct_builder: CStructBuilder

}

impl CCheckableStructBuilder {
    pub fn new(struct_builder: CStructBuilder) -> Self {
        Self {
            struct_builder,
            checked: None,
        }
    }
    pub fn checked(mut self, checked: bool) -> Self {
        self.checked = Some(checked);
        self
    }

    pub fn gui_name(mut self, name: String) -> Self {
        self.struct_builder = self.struct_builder.gui_name(name);
        self
    }

    pub fn build(self) -> CCheckableStruct {
        let mut r = CCheckableStruct::new(self.struct_builder.build());
        if let Some(checked) = self.checked {
            r.set_checked(checked);
        }
        r
    }
}
