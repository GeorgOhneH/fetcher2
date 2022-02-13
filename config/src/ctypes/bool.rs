use crate::errors::InValid;

#[cfg_attr(feature = "druid", derive(druid::Data))]
#[derive(Debug, Clone)]
pub struct CBool {
    pub(crate) value: Option<bool>,
    #[cfg_attr(feature = "druid", data(ignore))]
    pub(crate) name: Option<&'static str>,
}

impl CBool {
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
