use druid::{Data, Lens, Widget};
use druid::widget::{Label};

use crate::{InvalidError, State};


#[derive(Debug, Clone, Data, Lens)]
pub struct CFloat {
    value: Option<f64>,
    #[data(ignore)]
    min: f64,
    #[data(ignore)]
    max: f64,
    #[data(ignore)]
    name: Option<String>,
}

impl CFloat {
    fn new() -> Self {
        Self {
            value: None,
            min: f64::MIN,
            max: f64::MAX,
            name: None,
        }
    }

    pub fn is_valid(&self, value: &f64) -> Result<(), InvalidError> {
        if self.min <= *value && *value <= self.max {
            Ok(())
        } else {
            Err(InvalidError::new("Value must be between in bounds"))
        }
    }

    pub fn get(&self) -> Option<&f64> {
        Option::from(&self.value)
    }

    pub fn set_raw(&mut self, value: Option<f64>) -> Result<(), InvalidError> {
        if let Some(value) = value {
            self.set(value)
        } else {
            self.value = None;
            Ok(())
        }
    }

    pub fn set(&mut self, value: f64) -> Result<(), InvalidError> {
        self.is_valid(&value)?;
        self.value = Some(value);
        Ok(())
    }
    pub fn unset(&mut self) {
        self.value = None
    }

    pub fn state(&self) -> State {
        match &self.value {
            Some(v) => self.is_valid(v).into(),
            None => State::None,
        }
    }

    pub fn widget() -> impl Widget<Self> {
        Label::new("TODO")
        // Flex::row()
        //     .with_child(
        //         Maybe::or_empty(|| Label::dynamic(|data: &String, _| data.clone() + ":"))
        //             .lens(Self::name),
        //     )
        //     .with_child(Stepper::new().lens(Self::value))
    }
}

pub struct CFloatBuilder {
    inner: CFloat,
}

impl CFloatBuilder {
    pub fn new() -> Self {
        Self {
            inner: CFloat::new(),
        }
    }
    pub fn default(mut self, value: f64) -> Self {
        self.inner.set(value).unwrap();
        self
    }

    pub fn name(mut self, name: String) -> Self {
        self.inner.name = Some(name);
        self
    }

    pub fn max(mut self, max: f64) -> Self {
        if self.inner.value.is_some() && max < self.inner.value.unwrap() {
            panic!("Max smaller then value")
        }
        self.inner.max = max;
        self
    }
    pub fn min(mut self, min: f64) -> Self {
        if self.inner.value.is_some() && min > self.inner.value.unwrap() {
            panic!("Min bigger then value")
        }
        self.inner.min = min;
        self
    }
    pub fn build(self) -> CFloat {
        self.inner
    }
}
