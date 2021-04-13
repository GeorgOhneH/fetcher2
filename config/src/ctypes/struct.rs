use crate::*;

#[derive(Debug, Clone)]
pub struct CStruct {
    inner: HashMap<String, CKwarg>,
}

impl CStruct {
    fn new() -> Self {
        Self {
            inner: HashMap::new(),
        }
    }

    pub fn get(&self, name: &str) -> Option<&CKwarg> {
        self.inner.get(name)
    }

    pub fn get_mut(&mut self, name: &str) -> Option<&mut CKwarg> {
        self.inner.get_mut(name)
    }

    pub fn get_ty(&self, name: &str) -> Option<&CTypes> {
        match self.get(name) {
            Some(config_arg) => Some(config_arg.get()),
            None => None,
        }
    }

    pub fn get_ty_mut(&mut self, name: &str) -> Option<&mut CTypes> {
        match self.get_mut(name) {
            Some(config_arg) => Some(config_arg.get_mut()),
            None => None,
        }
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

#[derive(Debug, Clone)]
pub struct CKwarg {
    required: bool,
    name: String,
    gui_name: Option<String>,
    hint_text: Option<String>,
    active_fn: fn(CStruct) -> bool,
    inactive_behavior: InactiveBehavior,
    ty: CTypes,
}

impl CKwarg {
    fn new(name: String, ty: CTypes) -> Self {
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

    pub fn get(&self) -> &CTypes {
        &self.ty
    }

    pub fn get_mut(&mut self) -> &mut CTypes {
        &mut self.ty
    }
}

pub struct CKwargBuilder {
    inner: CKwarg,
}

impl CKwargBuilder {
    pub fn new(name: String, ty: CTypes) -> Self {
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