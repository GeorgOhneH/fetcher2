use crate::*;

#[derive(Debug, Clone)]
pub struct CInteger {
    value: Option<isize>,
    min: isize,
    max: isize,
}

impl CInteger {
    fn new() -> Self {
        Self {
            value: None,
            min: isize::MIN,
            max: isize::MAX,
        }
    }

    pub fn is_valid(&self, value: &isize) -> Result<(), InvalidError> {
        if self.min <= *value && *value <= self.max {
            Ok(())
        } else {
            Err(InvalidError::new(format!(
                "Value must be between {} and {}",
                self.min, self.max
            )))
        }
    }

    pub fn get(&self) -> Option<&isize> {
        Option::from(&self.value)
    }

    pub fn set(&mut self, value: isize) -> Result<(), InvalidError> {
        self.is_valid(&value)?;
        self.value = Some(value);
        Ok(())
    }
    pub fn unset(&mut self) {
        self.value = None
    }
}

pub struct CIntegerBuilder {
    inner: CInteger,
}

impl CIntegerBuilder {
    pub fn new() -> Self {
        Self {
            inner: CInteger::new(),
        }
    }
    pub fn default(mut self, value: isize) -> Self {
        self.inner.set(value).unwrap();
        self
    }

    pub fn max(mut self, max: isize) -> Self {
        if self.inner.value.is_some() && max < self.inner.value.unwrap() {
            panic!("Max smaller then value")
        }
        self.inner.max = max;
        self
    }
    pub fn min(mut self, min: isize) -> Self {
        if self.inner.value.is_some() && min > self.inner.value.unwrap() {
            panic!("Min bigger then value")
        }
        self.inner.min = min;
        self
    }
    pub fn build(self) -> CInteger {
        self.inner
    }
}
