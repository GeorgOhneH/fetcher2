use crate::{ConfigStruct, SupportedTypes};

#[derive(Debug, Clone)]
pub enum InactiveBehavior {
    GrayOut,
    Hide,
}

#[derive(Debug, Clone)]
pub struct ConfigArg {
    required: bool,
    name: String,
    gui_name: Option<String>,
    hint_text: Option<String>,
    active_fn: fn(ConfigStruct) -> bool,
    inactive_behavior: InactiveBehavior,
    ty: SupportedTypes,
}

impl ConfigArg {
    fn new(name: String, ty: SupportedTypes) -> Self {
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

    pub fn get(&self) -> &SupportedTypes {
        &self.ty
    }

    pub fn get_mut(&mut self) -> &mut SupportedTypes {
        &mut self.ty
    }
}

pub struct ConfigArgBuilder {
    inner: ConfigArg,
}

impl ConfigArgBuilder {
    pub fn new(name: String, ty: SupportedTypes) -> Self {
        Self {
            inner: ConfigArg::new(name, ty),
        }
    }

    pub fn required(mut self, required: bool) -> Self {
        self.inner.required = required;
        self
    }

    pub fn doc(mut self, hint_text: String) -> Self {
        self.inner.hint_text = Some(hint_text);
        self
    }

    pub fn inactive_behavior(mut self, behavior: InactiveBehavior) -> Self {
        self.inner.inactive_behavior = behavior;
        self
    }

    pub fn active_fn(mut self, active_fn: fn(ConfigStruct) -> bool) -> Self {
        self.inner.active_fn = active_fn;
        self
    }

    pub fn gui_name(mut self, name: String) -> Self {
        self.inner.gui_name = Some(name);
        self
    }
    pub fn build(self) -> ConfigArg {
        self.inner
    }
}
