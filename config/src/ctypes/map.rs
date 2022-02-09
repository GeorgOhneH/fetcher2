use std::hash::Hash;
use std::path::PathBuf;

use druid::im;
use druid::widget::{Label, ListIter};
use druid::{Data, Widget};
use crate::ctypes::CType;


#[derive(Debug, Clone, Data)]
pub struct CMap {
    pub inner: im::OrdMap<String, CType>,
    #[data(ignore)]
    pub value_template: Box<CType>,
    #[data(ignore)]
    name: Option<&'static str>,
}

impl CMap {
    pub fn new(value_template: CType) -> Self {
        Self {
            inner: im::OrdMap::new(),
            value_template: Box::new(value_template),
            name: None,
        }
    }

    pub fn widget() -> impl Widget<Self> {
        Label::new("Map can't really be used as a widget right now")
    }
}

impl ListIter<CType> for CMap {
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
