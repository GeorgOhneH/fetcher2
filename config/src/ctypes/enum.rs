use crate::*;
use serde_yaml::{Mapping, Value};
use std::process::id;

#[derive(Debug, Clone)]
pub struct CEnum {
    inner: HashMap<String, CArg>,
    selected: Option<String>,
}

impl CEnum {
    fn new() -> Self {
        Self {
            inner: HashMap::new(),
            selected: None,
        }
    }

    pub fn get_selected(&self) -> Option<&CArg> {
        match &self.selected {
            Some(idx) => Some(self.inner.get(idx).unwrap()),
            None => None,
        }
    }

    pub fn get_selected_mut(&mut self) -> Option<&mut CArg> {
        match &self.selected {
            Some(idx) => Some(self.inner.get_mut(idx).unwrap()),
            None => None,
        }
    }

    pub fn unselect(&mut self) {
        self.selected = None
    }

    pub fn set_selected(&mut self, idx: String) -> Result<&CArg, MsgError> {
        match self.inner.get(&idx) {
            Some(carg) => {
                self.selected = Some(idx);
                Ok(carg)
            }
            None => Err(MsgError::new("Key does not exist".to_string())),
        }
    }

    pub fn set_selected_mut(&mut self, idx: String) -> Result<&mut CArg, MsgError> {
        match self.inner.get_mut(&idx) {
            Some(carg) => {
                self.selected = Some(idx);
                Ok(carg)
            }
            None => Err(MsgError::new("Key does not exist".to_string())),
        }
    }

    pub(crate) fn consume_map(&mut self, map: Mapping) -> Result<(), RequiredError> {
        if map.len() != 1 {
            Err(RequiredError::new(
                "Enum map has the wrong format".to_owned(),
            ))
        } else if let Some((vkey, value)) = map.into_iter().next() {
            let key = match vkey {
                Value::String(str) => str,
                _ => return Err(RequiredError::new("map key is not String".to_owned())),
            };
            self.set_selected_mut(key)
                .map_err(|e| RequiredError::new("Key name does not exist".to_owned()))
                .and_then(|carg| carg.consume_value(value))
        } else {
            panic!("Should never happen")
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
        self.inner.inner.insert(carg.name.clone(), carg);
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
    parameter: Option<CStruct>,
}

impl CArg {
    fn new(name: String) -> Self {
        Self {
            name,
            gui_name: None,
            parameter: None,
        }
    }

    pub fn name(&self) -> &String {
        &self.name
    }

    pub fn get(&self) -> Option<&CStruct> {
        Option::from(&self.parameter)
    }

    pub fn get_mut(&mut self) -> Option<&mut CStruct> {
        Option::from(&mut self.parameter)
    }

    pub fn is_unit(&self) -> bool {
        self.parameter.is_none()
    }

    pub(crate) fn consume_value(&mut self, value: Value) -> Result<(), RequiredError> {
        match &mut self.parameter {
            Some(cstruct) => {
                if let Value::Mapping(map) = value {
                    cstruct.consume_map(map)
                } else {
                    Err(RequiredError::new("Struct Enum must be a Mapping".to_owned()))
                }

            },
            None => {
                if let Value::String(str) = value {
                    Ok(())
                } else {
                    Err(RequiredError::new("Unit Enum must be a String".to_owned()))
                }
            },
        }
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
        self.inner.parameter = Some(value);
        self
    }

    pub fn build(self) -> CArg {
        self.inner
    }
}
