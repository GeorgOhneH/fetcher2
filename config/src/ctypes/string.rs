use crate::ctypes::path::cpath_derived_lenses::value;
use crate::errors::InValid;

#[cfg_attr(feature = "druid", derive(druid::Data, druid::Lens))]
#[derive(Debug, Clone)]
pub struct CString {
    pub(crate) value: Option<String>,
    pub(crate) name: Option<&'static str>,
}

impl CString {
    pub fn new() -> Self {
        Self {
            value: None,
            name: None,
        }
    }

    pub fn valid(&self) -> Result<(), InValid> {
        if self.value.is_some() {
            Ok(())
        } else {
            Err(InValid::Required)
        }
    }

    pub fn set_name(&mut self, name: &'static str) {
        self.name = Some(name)
    }
}
