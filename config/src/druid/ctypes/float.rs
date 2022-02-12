use druid::widget::Label;
use druid::Widget;

use crate::ctypes::float::CFloat;

impl CFloat {
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
