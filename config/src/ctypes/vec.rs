use crate::{CTypes, MsgError, RequiredError};
use serde_yaml::Sequence;

#[derive(Debug, Clone)]
pub struct CVec {
    inner: Vec<CTypes>,
    template: Box<CTypes>,
}

impl CVec {
    fn new(template: CTypes) -> Self {
        Self {
            inner: Vec::new(),
            template: Box::new(template),
        }
    }

    pub fn get(&self) -> &Vec<CTypes> {
        &self.inner
    }

    pub fn get_template(&self) -> &CTypes {
        &self.template
    }

    pub fn is_valid(&self, vec: &Vec<CTypes>) -> Result<(), MsgError> {
        let r = vec
            .iter()
            .all(|ty| std::mem::discriminant(ty) == std::mem::discriminant(&self.template));
        if r {
            Ok(())
        } else {
            Err(MsgError::new(
                "SupportedTypes must be the same enum".to_string(),
            ))
        }
    }

    pub fn set(&mut self, vec: Vec<CTypes>) -> Result<(), MsgError> {
        if let Err(err) = self.is_valid(&vec) {
            Err(err)
        } else {
            self.inner = vec;
            Ok(())
        }
    }

    pub(crate) fn consume_sequence(&mut self, seq: Sequence) -> Result<(), RequiredError> {
        self.inner.clear();
        let mut result = Ok(());
        for value in seq {
            let mut template_copy = self.get_template().clone();
            match template_copy.consume_value(value) {
                Ok(()) => self.inner.push(template_copy),
                Err(err) => result = Err(err),
            }
        }
        result
    }
}

pub struct CVecBuilder {
    inner: CVec,
}

impl CVecBuilder {
    pub fn new(template: CTypes) -> Self {
        Self {
            inner: CVec::new(template),
        }
    }
    pub fn build(self) -> CVec {
        self.inner
    }
}
