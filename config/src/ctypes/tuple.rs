use im::Vector;
use crate::ctypes::CType;
use crate::errors::Error;


#[cfg_attr(feature = "druid", derive(druid::Data, druid::Lens))]
#[derive(Debug, Clone)]
pub struct CTuple {
    pub(crate)  inner: Vector<CType>,
    pub(crate) name: Option<&'static str>
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

