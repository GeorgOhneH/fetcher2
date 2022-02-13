use crate::ctypes::CType;
use crate::errors::InValid;

#[cfg_attr(feature = "druid", derive(druid::Data, druid::Lens))]
#[derive(Debug, Clone)]
pub struct COption {
    pub(crate) ty: CType,
    pub(crate) active: bool,
    pub(crate) name: Option<&'static str>,
}

impl COption {
    pub fn new(ty: CType) -> Self {
        Self {
            ty,
            active: true,
            name: None,
        }
    }

    pub fn valid(&self) -> Result<(), InValid> {
        match self.ty.valid() {
            Err(InValid::Required) => Ok(()),
            result => result,
        }
    }

    pub fn set_name(&mut self, name: &'static str) {
        self.name = Some(name)
    }
}
