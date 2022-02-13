use im::Vector;

use crate::ctypes::CType;
use crate::errors::{Error, InValid};

#[cfg_attr(feature = "druid", derive(druid::Data, druid::Lens))]
#[derive(Debug, Clone)]
pub struct CStruct {
    pub(crate) inner: Vector<CKwarg>,
    #[cfg_attr(feature = "druid", data(ignore))]
    pub(crate) index_map: im::OrdMap<&'static str, usize>,
    #[cfg_attr(feature = "druid", data(ignore))]
    pub(crate) name: Option<&'static str>,
}

impl CStruct {
    fn new() -> Self {
        Self {
            inner: Vector::new(),
            index_map: im::OrdMap::new(),
            name: None,
        }
    }

    pub fn valid(&self) -> Result<(), InValid> {
        Ok(())
    }

    pub fn set_name(&mut self, name: &'static str) {
        self.name = Some(name)
    }

    pub fn get(&self, name: &str) -> Result<&CKwarg, Error> {
        let idx = self.index_map.get(name).ok_or(Error::KeyDoesNotExist)?;
        Ok(&self.inner[*idx])
    }

    pub fn get_mut(&mut self, name: &str) -> Result<&mut CKwarg, Error> {
        let idx = self.index_map.get(name).ok_or(Error::KeyDoesNotExist)?;
        Ok(&mut self.inner[*idx])
    }

    pub fn get_idx_ty_mut(&mut self, idx: usize) -> Option<&mut CType> {
        self.inner.get_mut(idx).map(|kwarg| &mut kwarg.ty)
    }
}

pub struct CStructBuilder {
    inner: CStruct,
}

impl CStructBuilder {
    pub fn new() -> Self {
        Self {
            inner: CStruct::new(),
        }
    }
    pub fn arg(&mut self, arg: CKwarg) {
        let idx = self.inner.inner.len();
        self.inner.index_map.insert(arg.name, idx);
        self.inner.inner.push_back(arg);
    }

    pub fn build(self) -> CStruct {
        self.inner
    }
}

#[cfg_attr(feature = "druid", derive(druid::Data, druid::Lens))]
#[derive(Debug, Clone)]
pub struct CKwarg {
    #[cfg_attr(feature = "druid", data(ignore))]
    pub(crate) name: &'static str,
    #[cfg_attr(feature = "druid", data(ignore))]
    pub(crate) hint_text: Option<String>,
    pub(crate) ty: CType,
}

impl CKwarg {
    pub fn new(name: &'static str, ty: CType) -> Self {
        Self {
            ty,
            name,
            hint_text: None,
        }
    }
}
