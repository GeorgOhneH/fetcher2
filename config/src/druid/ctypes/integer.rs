use druid::widget::Label;
use druid::Widget;

use crate::ctypes::integer::CInteger;

impl CInteger {
    pub fn widget() -> impl Widget<Self> {
        Label::new("TODO")
        // Flex::row()
        //     .with_child(
        //         Maybe::or_empty(|| Label::dynamic(|data: &&'static str, _| format!("{data}:")))
        //             .lens(Self::name),
        //     )
        //     .with_child(IntStepper::new().lens(Self::value))
    }
}
