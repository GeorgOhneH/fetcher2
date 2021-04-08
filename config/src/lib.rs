#![allow(dead_code)]
use std::collections::HashMap;

#[derive(Debug)]
pub enum SupportedTypes {
    String(ConfigArgString),
    Bool(ConfigArgBool),
    Integer(ConfigArgInteger),
    Struct(Box<ConfigStruct>),
    Vec(Box<ConfigVec>),
}

#[derive(Debug)]
pub enum InactiveBehavior {
    GrayOut,
    Hide,
}

#[derive(Debug)]
pub struct ConfigStruct {
    inner: HashMap<String, ConfigArg>,
}

impl ConfigStruct {
    pub fn new() -> Self {
        Self {
            inner: HashMap::new(),
        }
    }
    pub fn arg(&mut self, arg: ConfigArg) -> &Self {
        self.inner.insert(arg.name.clone(), arg);
        self
    }

    pub fn get(&self, name: &String) -> Option<&ConfigArg> {
        self.inner.get(name)
    }

    pub fn get_ty(&self, name: &String) -> Option<&SupportedTypes> {
        match self.get(name) {
            Some(config_arg) => Some(&config_arg.ty),
            None => None,
        }
    }
}

#[derive(Debug)]
pub struct ConfigVec {
    inner: Vec<SupportedTypes>,
    template: SupportedTypes,
}

impl ConfigVec {
    pub fn new(template: SupportedTypes) -> Self {
        Self {
            inner: Vec::new(),
            template,
        }
    }
    pub fn push(&mut self, ty: SupportedTypes) -> &Self {
        self.inner.push(ty);
        self
    }
}

#[derive(Debug)]
pub struct ConfigArgString {
    value: Option<String>,
}

impl ConfigArgString {
    pub fn new() -> Self {
        Self { value: None }
    }
    pub fn default(mut self, value: String) -> Self {
        self.value = Some(value);
        self
    }
}

#[derive(Debug)]
pub struct ConfigArgBool {
    value: Option<bool>,
}

impl ConfigArgBool {
    pub fn new() -> Self {
        Self { value: None }
    }
    pub fn default(mut self, value: bool) -> Self {
        self.value = Some(value);
        self
    }
}

#[derive(Debug)]
pub struct ConfigArgInteger {
    value: Option<isize>,
    min: isize,
    max: isize,
}

impl ConfigArgInteger {
    pub fn new() -> Self {
        Self {
            value: None,
            min: isize::MAX,
            max: isize::MIN,
        }
    }
    pub fn default(mut self, value: isize) -> Self {
        self.value = Some(value);
        self
    }
    pub fn max(mut self, max: isize) -> Self {
        self.max = max;
        self
    }
    pub fn min(mut self, min: isize) -> Self {
        self.min = min;
        self
    }
}

#[derive(Debug)]
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
    pub fn new(name: String, ty: SupportedTypes) -> Self {
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

    pub fn gui_name(mut self, name: String) -> Self {
        self.gui_name = Some(name);
        self
    }

    pub fn inactive_behavior(mut self, behavior: InactiveBehavior) -> Self {
        self.inactive_behavior = behavior;
        self
    }

    pub fn active_fn(mut self, active_fn: fn(ConfigStruct) -> bool,) -> Self {
        self.active_fn = active_fn;
        self
    }

    pub fn is_required(&self) -> bool {
        self.required
    }

    pub fn required(mut self, required: bool) -> Self {
        self.required = required;
        self
    }

    pub fn doc(mut self, hint_text: String) -> Self {
        self.hint_text = Some(hint_text);
        self
    }

    pub fn get(&self) -> &SupportedTypes {
        &self.ty
    }

    pub fn get_mut(&mut self) -> &mut SupportedTypes {
        &mut self.ty
    }
}

pub trait Config {
    //fn parse() -> Self;
    fn build_app() -> ConfigStruct;
}

