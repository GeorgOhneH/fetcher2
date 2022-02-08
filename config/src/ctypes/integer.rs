use druid::widget::{Flex, Label, Maybe};
use druid::{Data, Lens, Widget, WidgetExt};

use crate::widgets::IntStepper;
use crate::errors::Error;

#[derive(Debug, Clone, Data, Lens)]
pub struct CInteger {
    pub value: Option<isize>,
    #[data(ignore)]
    min: isize,
    #[data(ignore)]
    max: isize,
    #[data(ignore)]
    name: Option<String>,
}

impl CInteger {
    pub fn new() -> Self {
        Self {
            value: None,
            min: isize::MIN,
            max: isize::MAX,
            name: None,
        }
    }

    pub fn is_valid(&self, value: &isize) -> Result<(), Error> {
        if self.min <= *value && *value <= self.max {
            Ok(())
        } else {
            Err(Error::IntOutOfRange {
                value: *value,
                min: self.min,
                max: self.max,
            })
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
