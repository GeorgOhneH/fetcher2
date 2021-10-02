use std::hash::Hash;
use std::path::PathBuf;

use druid::im;
use druid::widget::{Label, ListIter};
use druid::{Data, Widget};

use crate::{CType, State};

#[derive(Debug, Clone, Eq, Hash, PartialEq, PartialOrd, Ord, Data)]
pub enum HashKey {
    String(String),
    Path(#[data(same_fn = "PartialEq::eq")] PathBuf),
}

impl HashKey {
    pub fn set<T: Into<PathBuf>>(&mut self, name: T) {
        match self {
            Self::String(str) => *str = name.into().to_string_lossy().to_string(),
            Self::Path(path) => *path = name.into(),
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
            name: None,
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

    pub fn name(mut self, name: String) -> Self {
        self.inner.name = Some(name);
        self
    }

    pub fn build(self) -> CHashMap {
        self.inner
    }
}
