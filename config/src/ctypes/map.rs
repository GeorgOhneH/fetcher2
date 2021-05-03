use crate::{CType, InvalidError, ConfigError};
use lazy_static::lazy_static;
use serde_yaml::{Sequence, Mapping};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::hash::Hash;

#[derive(Debug, Clone, Eq, Hash, PartialEq)]
pub enum HashKey {
    String(String),
    Path(PathBuf),
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

    pub fn is_valid(&self, map: &HashMap<HashKey, CType>) -> Result<(), InvalidError> {
        Ok(())
    }

    pub fn set(&mut self, map: HashMap<HashKey, CType>) -> Result<(), InvalidError> {
        self.is_valid(&map)?;
        self.inner = map;
        Ok(())
    }

    pub(crate) fn consume_map(&mut self, map: Mapping) -> Result<(), ConfigError> {
        // TODO
        self.inner.clear();
        let mut result = Ok(());
        map.into_iter().map(|(k, v)| {

        });
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
