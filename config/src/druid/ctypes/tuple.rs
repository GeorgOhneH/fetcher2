use druid::{Data, Widget, WidgetExt, WidgetPod, Lens, Point};
use druid::im::Vector;
use druid::widget::prelude::*;
use druid::widget::{Checkbox, CrossAxisAlignment, Flex, Label};
use crate::ctypes::CType;
use crate::ctypes::tuple::CTuple;
use crate::errors::Error;

impl CTuple {
    pub fn widget() -> impl Widget<Self> {
        Label::new("TODO")
    }
}
