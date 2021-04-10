use crate::*;

#[derive(Debug, Clone)]
pub struct ConfigCheckableStruct {
    inner: ConfigStruct,
    checked: bool,
}

impl ConfigCheckableStruct {
    fn new(config_struct: ConfigStruct) -> Self {
        Self {
            inner: config_struct,
            checked: true,
        }
    }

    pub fn get_inner(&self) -> &ConfigStruct {
        &self.inner
    }

    pub fn get_inner_mut(&mut self) -> &mut ConfigStruct {
        &mut self.inner
    }

    pub fn set_checked(&mut self, checked: bool) {
        self.checked = checked;
    }

    pub fn is_checked(&self) -> bool {
        self.checked
    }
}

pub struct ConfigCheckableStructBuilder {
    inner: ConfigCheckableStruct,
}

impl ConfigCheckableStructBuilder {
    pub fn new(config_struct: ConfigStruct) -> Self {
        Self {
            inner: ConfigCheckableStruct::new(config_struct),
        }
    }
    pub fn checked(mut self, checked: bool) -> Self {
        self.inner.set_checked(checked);
        self
    }
    pub fn build(self) -> ConfigCheckableStruct {
        self.inner
    }
}
