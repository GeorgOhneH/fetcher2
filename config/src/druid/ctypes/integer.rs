use druid::widget::{Flex, Label, Maybe};
use druid::{Data, Lens, Widget, WidgetExt};
use serde::de::{Error as _, Visitor};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::fmt::{Formatter, Write};
use std::ops::{Deref, DerefMut};
use crate::ctypes::integer::CInteger;

use crate::errors::Error;
use crate::traveller::{Travel, Traveller};
use crate::druid::widgets::IntStepper;

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
