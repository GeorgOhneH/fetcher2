#[cfg_attr(feature = "druid", derive(druid::Data, druid::Lens))]
#[derive(Debug, Clone)]
pub struct CUnit {
    pub(crate) name: Option<&'static str>,
}

impl CUnit {
    pub fn new() -> Self {
        Self { name: None }
    }
}
