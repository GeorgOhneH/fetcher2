use crate::{CType, ConfigError, InvalidError, CArg, State};

use serde_yaml::{Mapping, Value};
use std::collections::HashMap;
use std::hash::Hash;
use std::path::{PathBuf};
use druid::{Data, Widget, WidgetExt};
use druid::im;
use druid::widget::{Flex, List, ListIter, Label};

#[derive(Debug, Clone, Eq, Hash, PartialEq, PartialOrd, Ord, Data)]
pub enum HashKey {
    String(String),
    Path(#[data(same_fn = "PartialEq::eq")] PathBuf),
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

#[derive(Debug, Clone, Data)]
pub struct CHashMap {
    inner: im::OrdMap<HashKey, CType>,
    #[data(ignore)]
    key_fn: fn() -> HashKey,
    #[data(ignore)]
    value_fn: fn() -> CType,
    #[data(ignore)]
    name: Option<String>,
}

impl CHashMap {
    fn new(key_fn: fn() -> HashKey, value_fn: fn() -> CType) -> Self {
        Self {
            inner: im::OrdMap::new(),
            key_fn,
            value_fn,
            name: None
        }
    }

    pub fn get(&self) -> &im::OrdMap<HashKey, CType> {
        &self.inner
    }

    pub fn get_key(&self) -> HashKey {
        (self.key_fn)()
    }

    pub fn get_value(&self) -> CType {
        (self.value_fn)()
    }

    pub fn set(&mut self, map: im::OrdMap<HashKey, CType>) {
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


    pub fn state(&self) -> State {
        self.inner.values().map(|ty| ty.state()).collect()
    }

    pub fn widget() -> impl Widget<Self> {
        Label::new("Map can't really be used as a widget right now")
    }
}

impl ListIter<CType> for CHashMap {
    fn for_each(&self, cb: impl FnMut(&CType, usize)) {
        self.inner.for_each(cb)
    }

    fn for_each_mut(&mut self, cb: impl FnMut(&mut CType, usize)) {
        self.inner.for_each_mut(cb)
    }

    fn data_len(&self) -> usize {
        self.inner.data_len()
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

    pub fn gui_name(mut self, name: String) -> Self {
        self.inner.name = Some(name);
        self
    }

    pub fn build(self) -> CHashMap {
        self.inner
    }
}
