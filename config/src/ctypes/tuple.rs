use druid::{Data, Widget, WidgetExt, WidgetPod, Lens, Point};
use druid::im::Vector;
use druid::widget::prelude::*;
use druid::widget::{Checkbox, CrossAxisAlignment, Flex, Label};
use crate::ctypes::CType;
use crate::errors::Error;

#[derive(Debug, Clone, Data, Lens)]
pub struct CTuple {
    pub inner: Vector<CType>,
    name: Option<&'static str>
}

impl CTuple {
    pub fn new() -> Self {
        Self {
            inner: Vector::new(),
            name: None,
        }
    }

    pub fn get(&self, idx: usize) -> Result<&CType, Error> {
        self.inner.get(idx).ok_or(Error::KeyDoesNotExist)
    }

    pub fn get_mut(&mut self, idx: usize) -> Result<&mut CType, Error> {
        self.inner.get_mut(idx).ok_or(Error::KeyDoesNotExist)
    }

    pub fn len(&self) -> usize {
        self.inner.len()
    }

    pub fn widget() -> impl Widget<Self> {
        Label::new("TODO")
    }
}

pub struct CTupleBuilder {
    inner: CTuple
}

impl CTupleBuilder {
    pub fn new() -> Self {
        Self {
            inner: CTuple::new()
        }
    }

    pub fn add_element(&mut self, ty: CType) {
        self.inner.inner.push_back(ty)
    }

    pub fn build(self) -> CTuple {
        self.inner
    }
}

