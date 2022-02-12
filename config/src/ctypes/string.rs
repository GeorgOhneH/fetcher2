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
}
