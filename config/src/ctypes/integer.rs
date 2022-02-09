use druid::widget::{Flex, Label, Maybe};
use druid::{Data, Lens, Widget, WidgetExt};
use serde::de::{Error as _, Visitor};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::fmt::{Formatter, Write};

use crate::errors::Error;
use crate::traveller::{Travel, Traveller};
use crate::widgets::IntStepper;

#[derive(Debug, Clone, Data, Lens)]
pub struct CInteger {
    pub value: Option<i64>,
    #[data(ignore)]
    min: i64,
    #[data(ignore)]
    max: i64,
    #[data(ignore)]
    name: Option<&'static str>,
}

impl CInteger {
    pub fn new(min: i64, max: i64) -> Self {
        Self {
            value: None,
            min,
            max,
            name: None,
        }
    }

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

#[derive(Debug)]
pub struct RangedInt<const MIN: i64, const MAX: i64>(pub i64);

impl<const MIN: i64, const MAX: i64> Travel for RangedInt<MIN, MAX> {
    fn travel<T>(traveller: T) -> Result<T::Ok, T::Error>
    where
        T: Traveller,
    {
        traveller.found_ranged_int(MIN, MAX)
    }
}

impl<const MIN: i64, const MAX: i64> Serialize for RangedInt<MIN, MAX> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_i64(self.0)
    }
}

impl<'de, const MIN: i64, const MAX: i64> Deserialize<'de> for RangedInt<MIN, MAX> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let v = i64::deserialize(deserializer)?;
        if v >= MIN && v <= MAX {
            Ok(RangedInt(v))
        } else {
            Err(D::Error::custom("Integer is not between {MIN} and {MAX}"))
        }
    }
}
