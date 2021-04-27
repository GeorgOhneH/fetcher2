use crate::{CTypes, InvalidError, ConfigError};
use lazy_static::lazy_static;
use serde_yaml::{Sequence, Mapping};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::hash::Hash;

pub enum HashKey {
    String(String),
    Path(PathBuf),
}

#[derive(Debug, Clone)]
pub struct CHashMap {
    inner: HashMap<HashKey, CTypes>,
    key_fn: fn() -> HashKey,
    value_fn: fn() -> CTypes,
}

impl CHashMap {
    fn new(key_fn: fn() -> HashKey, value_fn: fn() -> CTypes) -> Self {
        Self {
            inner: HashMap::new(),
            key_fn,
            value_fn,
        }
    }

    pub fn get(&self) -> &HashMap<HashKey, CTypes> {
        &self.inner
    }

    pub fn get_key(&self) -> HashKey {
        (self.key_fn)()
    }

    pub fn get_value(&self) -> CTypes {
        (self.value_fn)()
    }

    pub fn is_valid(&self, map: &HashMap<HashKey, CTypes>) -> Result<(), InvalidError> {
        Ok(())
    }

    pub fn set(&mut self, map: HashMap<HashKey, CTypes>) -> Result<(), InvalidError> {
        self.is_valid(&map)?;
        self.inner = map;
        Ok(())
    }

    pub(crate) fn consume_map(&mut self, map: Mapping) -> Result<(), ConfigError> {
        Ok(())
        // TODO
        // self.inner.clear();
        // let mut result = Ok(());
        // for value in seq {
        //     let mut template = self.get_template();
        //     match template.consume_value(value) {
        //         Ok(()) => self.inner.push(template),
        //         Err(err) => result = Err(err),
        //     }
        // }
        // result
    }
}

pub struct CHashMapBuilder {
    inner: CHashMap,
}

impl CHashMapBuilder {
    pub fn new(key_fn: fn() -> HashKey, value_fn: fn() -> CTypes) -> Self {
        Self {
            inner: CHashMap::new(key_fn, value_fn),
        }
    }
    pub fn build(self) -> CHashMap {
        self.inner
    }
}
