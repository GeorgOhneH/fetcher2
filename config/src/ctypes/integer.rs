use druid::widget::{Flex, Label, Maybe};
use druid::{Data, Lens, Widget, WidgetExt};

use crate::widgets::IntStepper;
use crate::{InvalidError, State};

#[derive(Debug, Clone, Data, Lens)]
pub struct CInteger {
    value: Option<isize>,
    #[data(ignore)]
    min: isize,
    #[data(ignore)]
    max: isize,
    #[data(ignore)]
    name: Option<String>,
}

impl CInteger {
    fn new() -> Self {
        Self {
            value: None,
            min: isize::MIN,
            max: isize::MAX,
            name: None,
        }
    }

    pub fn is_valid(&self, value: &isize) -> Result<(), InvalidError> {
        if self.min <= *value && *value <= self.max {
            Ok(())
        } else {
            Err(InvalidError::new("Value must be between in bounds"))
        }
    }

    pub fn get(&self) -> Option<&isize> {
        Option::from(&self.value)
    }

    pub fn set_raw(&mut self, value: Option<isize>) -> Result<(), InvalidError> {
        if let Some(value) = value {
            self.set(value)
        } else {
            self.value = None;
            Ok(())
        }
    }

    pub fn set(&mut self, value: isize) -> Result<(), InvalidError> {
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
        Flex::row()
            .with_child(
                Maybe::or_empty(|| Label::dynamic(|data: &String, _| data.clone() + ":"))
                    .lens(Self::name),
            )
            .with_child(IntStepper::new().lens(Self::value))
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

    pub fn name(mut self, name: String) -> Self {
        self.inner.name = Some(name);
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
