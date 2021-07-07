use crate::*;
use druid::{Data, Lens, Widget, WidgetExt};
use druid::widget::TextBox;
use druid::text::{ValidationError, Validation, Formatter, Selection};

#[derive(Debug, Clone, Data, Lens)]
pub struct CInteger {
    value: Option<isize>,
    #[data(ignore)]
    min: isize,
    #[data(ignore)]
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

    pub fn widget() -> impl Widget<Self> {
        TextBox::new()
            .with_formatter(IntFormatter::new())
            .update_data_while_editing(true)
            .lens(Self::value)
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

struct IntFormatter;

impl IntFormatter {
    pub fn new() -> Self {
        Self {}
    }
}

impl Formatter<Option<isize>> for IntFormatter {
    fn format(&self, value: &Option<isize>) -> String {
        match value {
            Some(v) => v.to_string(),
            None => "".to_owned(),
        }
    }
    fn format_for_editing(&self, value: &Option<isize>) -> String {
        self.format(value)
    }

    fn validate_partial_input(&self, input: &str, _sel: &Selection) -> Validation {
        match self.value(input) {
            Ok(_) => Validation::success(),
            Err(err) => Validation::failure(err),
        }
    }
    fn value(&self, input: &str) -> std::result::Result<Option<isize>, ValidationError> {
        if input.is_empty() {
            Ok(None)
        } else {
            input
                .parse()
                .map_err(ValidationError::new)
                .map(Option::Some)
        }
    }
}
