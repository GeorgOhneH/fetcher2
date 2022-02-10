use std::ops::{Deref, DerefMut};
use druid::widget::Label;
use druid::{Data, Lens, Widget};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use serde::de::Error as _;
use crate::ctypes::float::CFloat;
use crate::errors::Error;
use crate::traveller::{Travel, Traveller};

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

