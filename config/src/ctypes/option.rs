use crate::ctypes::CType;

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
}
