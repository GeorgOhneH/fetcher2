use crate::*;

#[derive(Debug, Clone)]
pub struct ConfigStruct {
    inner: HashMap<String, ConfigArg>,
}

impl ConfigStruct {
    fn new() -> Self {
        Self {
            inner: HashMap::new(),
        }
    }

    pub fn get(&self, name: &str) -> Option<&ConfigArg> {
        self.inner.get(name)
    }

    pub fn get_mut(&mut self, name: &str) -> Option<&mut ConfigArg> {
        self.inner.get_mut(name)
    }

    pub fn get_ty(&self, name: &str) -> Option<&SupportedTypes> {
        match self.get(name) {
            Some(config_arg) => Some(config_arg.get()),
            None => None,
        }
    }

    pub fn get_ty_mut(&mut self, name: &str) -> Option<&mut SupportedTypes> {
        match self.get_mut(name) {
            Some(config_arg) => Some(config_arg.get_mut()),
            None => None,
        }
    }
}

pub struct ConfigStructBuilder {
    inner: ConfigStruct,
}

impl ConfigStructBuilder {
    pub fn new() -> Self {
        Self {
            inner: ConfigStruct::new(),
        }
    }
    pub fn arg(mut self, arg: ConfigArg) -> Self {
        self.inner.inner.insert(arg.name().clone(), arg);
        self
    }
    pub fn build(self) -> ConfigStruct {
        self.inner
    }
}
