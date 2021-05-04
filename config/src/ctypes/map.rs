use crate::{CType, ConfigError, InvalidError};
use lazy_static::lazy_static;
use serde_yaml::{Mapping, Sequence, Value};
use std::collections::HashMap;
use std::hash::Hash;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Eq, Hash, PartialEq)]
pub enum HashKey {
    String(String),
    Path(PathBuf),
}

impl HashKey {
    fn consume_value(&mut self, value: Value) -> Result<(), ConfigError> {
        match self {
            HashKey::String(str) => match value {
                Value::String(vstr) => {
                    *str = vstr;
                    Ok(())
                }
                _ => Err(InvalidError::new("Expected String").into()),
            },
            HashKey::Path(path) => match value {
                Value::String(vstr) => {
                    *path = PathBuf::from(vstr);
                    Ok(())
                }
                _ => Err(InvalidError::new("Expected String").into()),
            },
        }
    }
}

#[derive(Debug, Clone)]
pub struct CHashMap {
    inner: HashMap<HashKey, CType>,
    key_fn: fn() -> HashKey,
    value_fn: fn() -> CType,
}

impl CHashMap {
    fn new(key_fn: fn() -> HashKey, value_fn: fn() -> CType) -> Self {
        Self {
            inner: HashMap::new(),
            key_fn,
            value_fn,
        }
    }

    pub fn get(&self) -> &HashMap<HashKey, CType> {
        &self.inner
    }

    pub fn get_key(&self) -> HashKey {
        (self.key_fn)()
    }

    pub fn get_value(&self) -> CType {
        (self.value_fn)()
    }

    pub fn set(&mut self, map: HashMap<HashKey, CType>) {
        self.inner = map;
    }

    pub(crate) fn consume_map(&mut self, map: Mapping) -> Result<(), ConfigError> {
        self.inner.clear();
        let mut result = Ok(());
        for (k, v) in map {
            let mut key = self.get_key();
            let mut value = self.get_value();
            if let Err(err) = value.consume_value(v) {
                result = Err(err);
                continue;
            }
            if let Err(err) = key.consume_value(k) {
                result = Err(err);
                continue;
            }
            self.inner.insert(key, value);
        }
        result
    }
}

pub struct CHashMapBuilder {
    inner: CHashMap,
}

impl CHashMapBuilder {
    pub fn new(key_fn: fn() -> HashKey, value_fn: fn() -> CType) -> Self {
        Self {
            inner: CHashMap::new(key_fn, value_fn),
        }
    }
    pub fn build(self) -> CHashMap {
        self.inner
    }
}
