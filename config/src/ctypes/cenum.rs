use im::{OrdMap, Vector};

use crate::ctypes::cstruct::CStruct;
use crate::ctypes::tuple::CTuple;
use crate::ctypes::CType;
use crate::errors::{Error, InValid};

#[cfg_attr(feature = "druid", derive(druid::Data, druid::Lens))]
#[derive(Debug, Clone)]
pub struct CEnum {
    pub(crate) inner: Vector<CArg>,
    pub(crate) index_map: OrdMap<&'static str, usize>,
    pub(crate) name_map: OrdMap<usize, &'static str>,
    pub(crate) selected: Option<usize>,
    pub(crate) name: Option<&'static str>,
}

impl CEnum {
    fn new() -> Self {
        Self {
            inner: Vector::new(),
            index_map: OrdMap::new(),
            name_map: OrdMap::new(),
            selected: None,
            name: None,
        }
    }

    pub fn valid(&self) -> Result<(), InValid> {
        if self.selected.is_some() {
            Ok(())
        } else {
            Err(InValid::Required)
        }
    }

    pub fn set_name(&mut self, name: &'static str) {
        self.name = Some(name)
    }

    pub fn get_selected(&self) -> Result<&CArg, Error> {
        self.selected
            .map(|idx| &self.inner[idx])
            .ok_or(Error::ValueRequired)
    }

    pub fn get_selected_mut(&mut self) -> Result<&mut CArg, Error> {
        self.selected
            .map(move |idx| &mut self.inner[idx])
            .ok_or(Error::ValueRequired)
    }

    pub fn set_selected(&mut self, variant: &str) -> Result<&CArg, Error> {
        match self.index_map.get(variant) {
            Some(i) => {
                self.selected = Some(*i);
                Ok(&self.inner[*i])
            }
            None => Err(Error::KeyDoesNotExist),
        }
    }

    pub fn set_selected_mut(&mut self, idx: &str) -> Result<&mut CArg, Error> {
        match self.index_map.get(idx) {
            Some(i) => {
                self.selected = Some(*i);
                Ok(&mut self.inner[*i])
            }
            None => Err(Error::KeyDoesNotExist),
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

    pub fn arg(&mut self, carg: CArg) {
        let idx = self.inner.inner.len();
        self.inner.index_map.insert(carg.name, idx);
        self.inner.name_map.insert(idx, carg.name);
        self.inner.inner.push_back(carg);
    }

    pub fn build(self) -> CEnum {
        self.inner
    }
}

#[cfg_attr(feature = "druid", derive(druid::Data))]
#[derive(Debug, Clone)]
pub enum CArgVariant {
    Unit,
    NewType(CType),
    Tuple(CTuple),
    Struct(CStruct),
}

impl CArgVariant {
    pub fn as_unit(&self) -> Result<(), Error> {
        match self {
            CArgVariant::Unit => Ok(()),
            _ => Err(Error::ExpectedUnitVariant),
        }
    }

    pub fn as_new_type(&self) -> Result<&CType, Error> {
        match self {
            CArgVariant::NewType(ty) => Ok(ty),
            _ => Err(Error::ExpectedNewTypeVariant),
        }
    }

    pub fn as_new_type_mut(&mut self) -> Result<&mut CType, Error> {
        match self {
            CArgVariant::NewType(ty) => Ok(ty),
            _ => Err(Error::ExpectedNewTypeVariant),
        }
    }

    pub fn as_tuple(&self) -> Result<&CTuple, Error> {
        match self {
            CArgVariant::Tuple(ctuple) => Ok(ctuple),
            _ => Err(Error::ExpectedTupleVariant),
        }
    }

    pub fn as_tuple_mut(&mut self) -> Result<&mut CTuple, Error> {
        match self {
            CArgVariant::Tuple(ctuple) => Ok(ctuple),
            _ => Err(Error::ExpectedTupleVariant),
        }
    }

    pub fn as_struct(&self) -> Result<&CStruct, Error> {
        match self {
            CArgVariant::Struct(cstruct) => Ok(cstruct),
            _ => Err(Error::ExpectedStructVariant),
        }
    }

    pub fn as_struct_mut(&mut self) -> Result<&mut CStruct, Error> {
        match self {
            CArgVariant::Struct(cstruct) => Ok(cstruct),
            _ => Err(Error::ExpectedStructVariant),
        }
    }
}

#[cfg_attr(feature = "druid", derive(druid::Data, druid::Lens))]
#[derive(Debug, Clone)]
pub struct CArg {
    #[cfg_attr(feature = "druid", data(ignore))]
    pub name: &'static str,

    pub variant: CArgVariant,
}

impl CArg {
    pub fn new(name: &'static str, variant: CArgVariant) -> Self {
        Self { name, variant }
    }
}
