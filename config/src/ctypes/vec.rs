use crate::{CTypes, InvalidError, ConfigError};
use lazy_static::lazy_static;
use serde_yaml::Sequence;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct CVec {
    inner: Vec<CTypes>,
    template_fn: fn() -> CTypes,
}

impl CVec {
    fn new(template_fn: fn() -> CTypes) -> Self {
        Self {
            inner: Vec::new(),
            template_fn,
        }
    }

    pub fn get(&self) -> &Vec<CTypes> {
        &self.inner
    }

    pub fn get_template(&self) -> CTypes {
        (self.template_fn)()
    }

    pub fn is_valid(&self, vec: &Vec<CTypes>) -> Result<(), InvalidError> {
        let template = self.get_template();
        let r = vec
            .iter()
            .all(|ty| std::mem::discriminant(ty) == std::mem::discriminant(&template));
        if r {
            Ok(())
        } else {
            Err(InvalidError::new("SupportedTypes must be the same enum"))
        }
    }

    pub fn set(&mut self, vec: Vec<CTypes>) -> Result<(), InvalidError> {
       self.is_valid(&vec)?;
        self.inner = vec;
        Ok(())
    }

    pub(crate) fn consume_sequence(&mut self, seq: Sequence) -> Result<(), ConfigError> {
        self.inner.clear();
        let mut result = Ok(());
        for value in seq {
            let mut template = self.get_template();
            match template.consume_value(value) {
                Ok(()) => self.inner.push(template),
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
    pub fn new(template: fn() -> CTypes) -> Self {
        Self {
            inner: CVec::new(template),
        }
    }
    pub fn build(self) -> CVec {
        self.inner
    }
}
