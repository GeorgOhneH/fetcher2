use crate::{SupportedTypes, ValueError};

#[derive(Debug, Clone)]
pub struct ConfigVec {
    inner: Vec<SupportedTypes>,
    template: SupportedTypes,
}

impl ConfigVec {
    fn new(template: SupportedTypes) -> Self {
        Self {
            inner: Vec::new(),
            template,
        }
    }

    pub fn get(&self) -> &Vec<SupportedTypes> {
        &self.inner
    }

    pub fn get_template(&self) -> &SupportedTypes {
        &self.template
    }

    pub fn is_valid(&self, vec: &Vec<SupportedTypes>) -> Result<(), ValueError> {
        let r = vec
            .iter()
            .all(|ty| std::mem::discriminant(ty) == std::mem::discriminant(&self.template));
        if r {
            Ok(())
        } else {
            Err(ValueError::new(
                "SupportedTypes must be the same enum".to_string(),
            ))
        }
    }

    pub fn set(&mut self, vec: Vec<SupportedTypes>) -> Result<(), ValueError> {
        if let Err(err) = self.is_valid(&vec) {
            Err(err)
        } else {
            self.inner = vec;
            Ok(())
        }
    }
}

pub struct ConfigVecBuilder {
    inner: ConfigVec,
}

impl ConfigVecBuilder {
    pub fn new(template: SupportedTypes) -> Self {
        Self {
            inner: ConfigVec::new(template),
        }
    }
    pub fn build(self) -> ConfigVec {
        self.inner
    }
}