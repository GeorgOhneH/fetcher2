use druid::widget::Label;
use druid::{Data, Lens, Widget};
use crate::errors::Error;

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

    pub fn is_valid(&self, value: &f64) -> Result<(), Error> {
        if self.min <= *value && *value <= self.max {
            Ok(())
        } else {
            Err(Error::FloatOutOfRange {
                value: *value,
                min: self.min,
                max: self.max
            })
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
