use crate::*;

#[derive(Debug, Clone)]
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
}

pub struct CCheckableStructBuilder {
    inner: CCheckableStruct,
}

impl CCheckableStructBuilder {
    pub fn new(config_struct: CStruct) -> Self {
        Self {
            inner: CCheckableStruct::new(config_struct),
        }
    }
    pub fn checked(mut self, checked: bool) -> Self {
        self.inner.set_checked(checked);
        self
    }
    pub fn build(self) -> CCheckableStruct {
        self.inner
    }
}
