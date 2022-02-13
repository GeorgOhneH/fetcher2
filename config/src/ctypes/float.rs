use std::ops::{Deref, DerefMut};

use serde::de::Error as _;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

use crate::errors::InValid;
use crate::traveller::{Travel, Traveller};

#[cfg_attr(feature = "druid", derive(druid::Data, druid::Lens))]
#[derive(Debug, Clone)]
pub struct CFloat {
    pub(crate) value: Option<f64>,
    #[cfg_attr(feature = "druid", data(ignore))]
    pub(crate) min: f64,
    #[cfg_attr(feature = "druid", data(ignore))]
    pub(crate) max: f64,
    #[cfg_attr(feature = "druid", data(ignore))]
    pub(crate) name: Option<&'static str>,
}

impl CFloat {
    pub fn new(min: f64, max: f64) -> Self {
        Self {
            value: None,
            min,
            max,
            name: None,
        }
    }

    pub fn valid(&self) -> Result<(), InValid> {
        if self.value.is_some() {
            Ok(())
        } else {
            Err(InValid::Required)
        }
    }

    pub fn set_name(&mut self, name: &'static str) {
        self.name = Some(name)
    }
}

#[derive(Debug)]
pub struct RangedFloat<const MIN: f64, const MAX: f64>(pub f64);

impl<const MIN: f64, const MAX: f64> Deref for RangedFloat<MIN, MAX> {
    type Target = f64;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<const MIN: f64, const MAX: f64> DerefMut for RangedFloat<MIN, MAX> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<const MIN: f64, const MAX: f64> Travel for RangedFloat<MIN, MAX> {
    fn travel<T>(traveller: T) -> Result<T::Ok, T::Error>
    where
        T: Traveller,
    {
        traveller.found_ranged_float(MIN, MAX)
    }
}

impl<const MIN: f64, const MAX: f64> Serialize for RangedFloat<MIN, MAX> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_f64(self.0)
    }
}

impl<'de, const MIN: f64, const MAX: f64> Deserialize<'de> for RangedFloat<MIN, MAX> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let v = f64::deserialize(deserializer)?;
        if v >= MIN && v <= MAX {
            Ok(RangedFloat(v))
        } else {
            Err(D::Error::custom(format!(
                "Float is not between {MIN} and {MAX}"
            )))
        }
    }
}
