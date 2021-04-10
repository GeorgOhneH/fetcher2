use crate::*;

#[derive(Debug, Clone)]
pub struct ConfigArgInteger {
    value: Option<isize>,
    min: isize,
    max: isize,
}

impl ConfigArgInteger {
    fn new() -> Self {
        Self {
            value: None,
            min: isize::MIN,
            max: isize::MAX,
        }
    }

    pub fn is_valid(&self, value: &Option<isize>) -> Result<(), ValueError> {
        match value {
            None => Ok(()),
            Some(int) if self.min <= *int && *int <= self.max => Ok(()),
            _ => Err(ValueError::new(format!(
                "Value must be between {} and {}",
                self.min, self.max
            ))),
        }
    }

    pub fn get(&self) -> Option<&isize> {
        Option::from(&self.value)
    }

    pub fn set(&mut self, value: Option<isize>) -> Result<(), ValueError> {
        if let Err(err) = self.is_valid(&value) {
            Err(err)
        } else {
            self.value = value;
            Ok(())
        }
    }
}

pub struct ConfigArgIntegerBuilder {
    inner: ConfigArgInteger,
}

impl ConfigArgIntegerBuilder {
    pub fn new() -> Self {
        Self {
            inner: ConfigArgInteger::new(),
        }
    }
    pub fn default(mut self, value: isize) -> Self {
        self.inner.set(Some(value)).unwrap();
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
    pub fn build(self) -> ConfigArgInteger {
        self.inner
    }
}
