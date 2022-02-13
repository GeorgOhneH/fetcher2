use crate::ctypes::CType;
use crate::errors::InValid;

#[cfg_attr(feature = "druid", derive(druid::Data, druid::Lens))]
#[derive(Debug, Clone)]
pub struct CMap {
    pub(crate) inner: im::OrdMap<String, CType>,
    #[cfg_attr(feature = "druid", data(ignore))]
    pub(crate) value_template: Box<CType>,
    #[cfg_attr(feature = "druid", data(ignore))]
    pub(crate) name: Option<&'static str>,
}

impl CMap {
    pub fn new(value_template: CType) -> Self {
        Self {
            inner: im::OrdMap::new(),
            value_template: Box::new(value_template),
            name: None,
        }
    }

    pub fn valid(&self) -> Result<(), InValid> {
        Ok(())
    }

    pub fn set_name(&mut self, name: &'static str) {
        self.name = Some(name)
    }
}
