use crate::*;
use serde_yaml::{Mapping, Value};

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
        self.selected
            .as_ref()
            .map(|idx| self.inner.get(idx).unwrap())
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

    pub fn set_selected(&mut self, idx: String) -> Result<&CArg, InvalidError> {
        match self.inner.get(&idx) {
            Some(carg) => {
                self.selected = Some(idx);
                Ok(carg)
            }
            None => Err(InvalidError::new("Key does not exist")),
        }
    }

    pub fn set_selected_mut(&mut self, idx: String) -> Result<&mut CArg, InvalidError> {
        match self.inner.get_mut(&idx) {
            Some(carg) => {
                self.selected = Some(idx);
                Ok(carg)
            }
            None => Err(InvalidError::new("Key does not exist")),
        }
    }

    pub(crate) fn consume_map(&mut self, map: Mapping) -> Result<(), ConfigError> {
        if map.len() != 1 {
            Err(InvalidError::new("Enum map has the wrong format").into())
        } else if let Some((vkey, value)) = map.into_iter().next() {
            let key = match vkey {
                Value::String(str) => str,
                _ => return Err(InvalidError::new("map key is not String").into()),
            };
            let mut carg = self.set_selected_mut(key)?;
            carg.consume_value(value)
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
    parameter: Option<CType>,
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

    pub fn get(&self) -> Option<&CType> {
        Option::from(&self.parameter)
    }

    pub fn get_mut(&mut self) -> Option<&mut CType> {
        Option::from(&mut self.parameter)
    }

    pub fn is_unit(&self) -> bool {
        self.parameter.is_none()
    }

    pub(crate) fn consume_value(&mut self, value: Value) -> Result<(), ConfigError> {
        match &mut self.parameter {
            Some(ctype) => ctype.consume_value(value),
            None => {
                if let Value::String(_) = value {
                    Ok(())
                } else {
                    Err(InvalidError::new("Unit Enum must be a String").into())
                }
            }
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

    pub fn value(mut self, value: CType) -> Self {
        self.inner.parameter = Some(value);
        self
    }

    pub fn build(self) -> CArg {
        self.inner
    }
}
