use crate::*;
use druid::im;
use druid::widget::{Flex, List, ListIter};
use druid::{Data, Lens, Widget, WidgetExt};
use serde_yaml::{Mapping, Value};
use std::collections::hash_map::Iter;

#[derive(Debug, Clone, Data)]
pub struct CStruct {
    inner: im::OrdMap<String, CKwarg>,
}

impl CStruct {
    fn new() -> Self {
        Self {
            inner: im::OrdMap::new(),
        }
    }

    pub fn get(&self, name: &str) -> Option<&CKwarg> {
        self.inner.get(name)
    }

    pub fn get_mut(&mut self, name: &str) -> Option<&mut CKwarg> {
        self.inner.get_mut(name)
    }

    pub fn get_ty(&self, name: &str) -> Option<&CType> {
        match self.get(name) {
            Some(config_arg) => Some(config_arg.get()),
            None => None,
        }
    }

    pub fn get_ty_mut(&mut self, name: &str) -> Option<&mut CType> {
        match self.get_mut(name) {
            Some(config_arg) => Some(config_arg.get_mut()),
            None => None,
        }
    }

    pub fn load_from_string(&mut self, str: &str) -> Result<(), ConfigError> {
        let value = serde_yaml::from_str::<Value>(&str)?;
        if let Value::Mapping(map) = value {
            self.consume_map(map)
        } else {
            Err(RequiredError::new("Root", "Must be a mapping").into())
        }
    }

    pub fn iter(&self) -> im::ordmap::Iter<String, CKwarg> {
        self.inner.iter()
    }

    pub(crate) fn consume_map(&mut self, mut map: Mapping) -> Result<(), ConfigError> {
        let mut result = Ok(());
        for (key, ckwarg) in self.inner.clone().iter() {
            match map.remove(&Value::String(key.to_string())) {
                Some(value) => {
                    let mut kwarg_clone = ckwarg.clone();
                    match kwarg_clone.consume_value(value) {
                        Ok(()) => self.inner[key] = kwarg_clone,
                        Err(err) => result = Err(err),
                    }
                }
                None => result = Err(RequiredError::new(key, "Missing value(s)").into()),
            }
        }
        result
    }

    pub fn widget() -> impl Widget<Self> {
        List::new(|| CKwarg::widget())
    }
}

impl ListIter<CKwarg> for CStruct {
    fn for_each(&self, cb: impl FnMut(&CKwarg, usize)) {
        self.inner.for_each(cb)
    }

    fn for_each_mut(&mut self, cb: impl FnMut(&mut CKwarg, usize)) {
        self.inner.for_each_mut(cb)
    }

    fn data_len(&self) -> usize {
        self.inner.data_len()
    }
}

pub struct CStructBuilder {
    inner: CStruct,
}

impl CStructBuilder {
    pub fn new() -> Self {
        Self {
            inner: CStruct::new(),
        }
    }
    pub fn arg(mut self, arg: CKwarg) -> Self {
        self.inner.inner.insert(arg.name().clone(), arg);
        self
    }
    pub fn build(self) -> CStruct {
        self.inner
    }
}

#[derive(Debug, Clone)]
pub enum InactiveBehavior {
    GrayOut,
    Hide,
}

#[derive(Debug, Clone, Data, Lens)]
pub struct CKwarg {
    #[data(ignore)]
    required: bool,
    #[data(ignore)]
    #[lens(name = "name_lens")]
    name: String,
    #[data(ignore)]
    gui_name: Option<String>,
    #[data(ignore)]
    hint_text: Option<String>,
    #[data(ignore)]
    active_fn: fn(CStruct) -> bool,
    #[data(ignore)]
    inactive_behavior: InactiveBehavior,
    ty: CType,
}

impl CKwarg {
    fn new(name: String, ty: CType) -> Self {
        Self {
            ty,
            name,
            gui_name: None,
            active_fn: |_app| true,
            hint_text: None,
            inactive_behavior: InactiveBehavior::GrayOut,
            required: true,
        }
    }

    pub fn name(&self) -> &String {
        &self.name
    }

    pub fn is_required(&self) -> bool {
        self.required
    }

    pub fn get(&self) -> &CType {
        &self.ty
    }

    pub fn get_mut(&mut self) -> &mut CType {
        &mut self.ty
    }

    pub fn consume_value(&mut self, value: Value) -> Result<(), ConfigError> {
        self.ty.consume_value(value)
    }

    fn widget() -> impl Widget<Self> {
        Flex::column().with_child(CType::widget().lens(Self::ty))
    }
}

pub struct CKwargBuilder {
    inner: CKwarg,
}

impl CKwargBuilder {
    pub fn new(name: String, ty: CType) -> Self {
        Self {
            inner: CKwarg::new(name, ty),
        }
    }

    pub fn required(mut self, required: bool) -> Self {
        self.inner.required = required;
        self
    }

    pub fn hint_text(mut self, hint_text: String) -> Self {
        self.inner.hint_text = Some(hint_text);
        self
    }

    pub fn inactive_behavior(mut self, behavior: InactiveBehavior) -> Self {
        self.inner.inactive_behavior = behavior;
        self
    }

    pub fn active_fn(mut self, active_fn: fn(CStruct) -> bool) -> Self {
        self.inner.active_fn = active_fn;
        self
    }

    pub fn gui_name(mut self, name: String) -> Self {
        self.inner.gui_name = Some(name);
        self
    }
    pub fn build(self) -> CKwarg {
        self.inner
    }
}
