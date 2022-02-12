use druid::widget::Label;
use druid::Widget;

use crate::ctypes::tuple::CTuple;

impl CTuple {
    pub fn widget() -> impl Widget<Self> {
        Label::new("TODO")
    }
}
