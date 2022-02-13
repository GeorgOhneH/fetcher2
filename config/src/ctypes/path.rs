use std::marker::PhantomData;
use std::ops::{Deref, DerefMut};
use std::path::PathBuf;

use im::Vector;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

use crate::errors::InValid;
use crate::traveller::{Travel, TravelPathConfig, Traveller};

#[cfg_attr(feature = "druid", derive(druid::Data, druid::Lens))]
#[derive(Debug, Clone)]
pub struct CPath {
    #[cfg_attr(feature = "druid", data(same_fn = "PartialEq::eq"))]
    pub(crate) value: Option<PathBuf>,
    #[cfg_attr(feature = "druid", data(ignore))]
    pub(crate) path_config: TravelPathConfig,
    #[cfg_attr(feature = "druid", data(ignore))]
    pub(crate) name: Option<&'static str>,
}

impl CPath {
    pub fn new(path_config: TravelPathConfig) -> Self {
        Self {
            value: None,
            path_config,
            name: None,
        }
    }

    pub fn set_name(&mut self, name: &'static str) {
        self.name = Some(name)
    }

    pub fn valid(&self) -> Result<(), InValid> {
        match &self.value {
            None => Err(InValid::Required),
            Some(v) => self.path_config.is_valid(v.as_path()),
        }
    }
}

pub trait PathConfig {
    fn config() -> TravelPathConfig;
}

#[cfg_attr(feature = "druid", derive(druid::Data))]
#[derive(Debug, Clone)]
pub struct AnyPath;

#[cfg_attr(feature = "druid", derive(druid::Data))]
#[derive(Debug, Clone)]
pub struct Relative;

#[cfg_attr(feature = "druid", derive(druid::Data))]
#[derive(Debug, Clone)]
pub struct Absolute;

#[cfg_attr(feature = "druid", derive(druid::Data))]
#[derive(Debug, Clone)]
pub struct AbsoluteExist;

#[cfg_attr(feature = "druid", derive(druid::Data))]
#[derive(Debug, Clone)]
pub struct AbsoluteExistFile;

#[cfg_attr(feature = "druid", derive(druid::Data))]
#[derive(Debug, Clone)]
pub struct AbsoluteExistDir;

impl PathConfig for AnyPath {
    fn config() -> TravelPathConfig {
        TravelPathConfig::Any
    }
}

impl PathConfig for Relative {
    fn config() -> TravelPathConfig {
        TravelPathConfig::Relative
    }
}

impl PathConfig for Absolute {
    fn config() -> TravelPathConfig {
        TravelPathConfig::Absolute
    }
}

impl PathConfig for AbsoluteExist {
    fn config() -> TravelPathConfig {
        TravelPathConfig::AbsoluteExist
    }
}

impl PathConfig for AbsoluteExistFile {
    fn config() -> TravelPathConfig {
        TravelPathConfig::AbsoluteExistFile(Vector::new())
    }
}

impl PathConfig for AbsoluteExistDir {
    fn config() -> TravelPathConfig {
        TravelPathConfig::AbsoluteExistDir
    }
}

#[cfg_attr(feature = "druid", derive(druid::Data))]
#[derive(Debug, Clone)]
pub struct StrictPath<CONFIG>
where
    CONFIG: PathConfig,
{
    #[cfg_attr(feature = "druid", data(eq))]
    path: PathBuf,
    _m0: PhantomData<CONFIG>,
}

impl<CONFIG> StrictPath<CONFIG>
where
    CONFIG: PathConfig,
{
    pub fn new() -> Self {
        Self {
            path: PathBuf::new(),
            _m0: PhantomData,
        }
    }
}

impl<T, CONFIG> From<T> for StrictPath<CONFIG>
where
    CONFIG: PathConfig,
    PathBuf: From<T>,
{
    fn from(v: T) -> Self {
        Self {
            path: PathBuf::from(v),
            _m0: PhantomData,
        }
    }
}

impl<CONFIG> Deref for StrictPath<CONFIG>
where
    CONFIG: PathConfig,
{
    type Target = PathBuf;

    fn deref(&self) -> &Self::Target {
        &self.path
    }
}

impl<CONFIG> DerefMut for StrictPath<CONFIG>
where
    CONFIG: PathConfig,
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.path
    }
}

impl<CONFIG> Travel for StrictPath<CONFIG>
where
    CONFIG: PathConfig,
{
    fn travel<T>(traveller: T) -> Result<T::Ok, T::Error>
    where
        T: Traveller,
    {
        traveller.found_path(CONFIG::config())
    }
}

impl<CONFIG> Serialize for StrictPath<CONFIG>
where
    CONFIG: PathConfig,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        self.path.serialize(serializer)
    }
}

impl<'de, CONFIG> Deserialize<'de> for StrictPath<CONFIG>
where
    CONFIG: PathConfig,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let path = PathBuf::deserialize(deserializer)?;
        CONFIG::config().is_valid(&path)?;
        Ok(Self {
            path,
            _m0: PhantomData,
        })
    }
}

impl<CONFIG> PartialEq for StrictPath<CONFIG>
where
    CONFIG: PathConfig,
{
    fn eq(&self, other: &Self) -> bool {
        self.path == other.path
    }
}
