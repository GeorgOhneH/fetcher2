use crate::*;
use std::ffi::CStr;
use std::process::id;

#[derive(Debug, Clone)]
pub struct CEnum {
    inner: Vec<CArg>,
    selected: Option<usize>,
}

impl CEnum {
    fn new() -> Self {
        Self {
            inner: Vec::new(),
            selected: None
        }
    }

    pub fn get_selected(&self) -> Option<&CArg> {
        match self.selected {
            Some(idx) => Some(&self.inner[idx]),
            None => None,
        }
    }

    pub fn get_selected_mut(&mut self) -> Option<&mut CArg> {
        match self.selected {
            Some(idx) => Some(&mut self.inner[idx]),
            None => None,
        }
    }

    pub fn set_selected(&mut self, idx: usize) -> Result<&CArg, MsgError> {
        if idx >= self.inner.len() {
            Err(MsgError::new("Index out of range".to_string()))
        } else {
            self.selected = Some(idx);
            Ok(&self.inner[idx])
        }
    }

    pub fn set_selected_mut(&mut self, idx: usize) -> Result<&mut CArg, MsgError> {
        if idx >= self.inner.len() {
            Err(MsgError::new("Index out of range".to_string()))
        } else {
            self.selected = Some(idx);
            Ok(&mut self.inner[idx])
        }
    }
}

pub struct CEnumBuilder {
    inner: CEnum,
}

impl CEnumBuilder {
    pub fn new() -> Self {
        Self {
            inner: CEnum::new(),
        }
    }

    pub fn arg(mut self, carg: CArg) -> Self {
        self.inner.inner.push(carg);
        self
    }

    pub fn build(self) -> CEnum {
        self.inner
    }
}

#[derive(Debug, Clone)]
pub struct CArg {
    name: String,
    gui_name: Option<String>,
    value: Option<CStruct>,
}

impl CArg {
    fn new(name: String) -> Self {
        Self {
            name,
            gui_name: None,
            value: None,
        }
    }

    pub fn name(&self) -> &String {
        &self.name
    }

    pub fn get(&self) -> Option<&CStruct> {
        Option::from(&self.value)
    }

    pub fn get_mut(&mut self) -> Option<&mut CStruct> {
        Option::from(&mut self.value)
    }
}

pub struct CArgBuilder {
    inner: CArg,
}

impl CArgBuilder {
    pub fn new(name: String) -> Self {
        Self {
            inner: CArg::new(name),
        }
    }

    pub fn gui_name(mut self, name: String) -> Self {
        self.inner.gui_name = Some(name);
        self
    }

    pub fn value(mut self, value: CStruct) -> Self {
        self.inner.value = Some(value);
        self
    }

    pub fn build(self) -> CArg {
        self.inner
    }
}
