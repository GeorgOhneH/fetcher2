use std::marker::PhantomData;
use std::ops::{Deref, DerefMut};
use std::path::{Path, PathBuf};

use im::Vector;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

use crate::errors::Error;
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

    pub fn is_valid(&self, _path: &Path) -> Result<(), Error> {
        todo!()
    }
}

pub trait PathConfig {
    fn config() -> TravelPathConfig;
}

#[derive(Debug, Clone)]
pub struct AnyPath;
#[derive(Debug, Clone)]
pub struct Relative;
#[derive(Debug, Clone)]
pub struct Absolute;
#[derive(Debug, Clone)]
pub struct AbsoluteExist;
#[derive(Debug, Clone)]
pub struct AbsoluteExistFile;
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

    pub fn is_valid<E: serde::de::Error>(
        path: &Path,
        path_config: &TravelPathConfig,
    ) -> Result<(), E> {
        match path_config {
            TravelPathConfig::Any => (),
            TravelPathConfig::Relative => {
                if !path.is_relative() {
                    return Err(E::custom("Expected relative Path"));
                }
            }
            TravelPathConfig::Absolute => {
                if !path.is_absolute() {
                    return Err(E::custom("Expected absolute Path"));
                }
            }
            TravelPathConfig::AbsoluteExist
            | TravelPathConfig::AbsoluteExistFile(_)
            | TravelPathConfig::AbsoluteExistDir => {
                if !path.is_absolute() {
                    return Err(E::custom("Expected absolute Path"));
                }
                if let Ok(metadata) = path.metadata() {
                    match path_config {
                        TravelPathConfig::AbsoluteExistDir => {
                            if !metadata.is_dir() {
                                return Err(E::custom("Expected a directory"));
                            }
                        }
                        TravelPathConfig::AbsoluteExistFile(extensions) => {
                            if metadata.is_file() {
                                return Err(E::custom("Expected a file"));
                            }

                            if !extensions.is_empty() {
                                if let Some(file_extension) = path.extension() {
                                    let ex = file_extension
                                        .to_str()
                                        .ok_or_else(|| E::custom("Expected Path with extension"))?;
                                    if !extensions
                                        .iter()
                                        .any(|file_spec| file_spec.extensions.contains(&ex))
                                    {
                                        return Err(E::custom("Path extension didn't match"));
                                    }
                                } else {
                                    return Err(E::custom("Expected Path with extension"));
                                }
                            }
                        }
                        _ => (),
                    }
                } else {
                    return Err(E::custom("Expected a existing Path"));
                }
            }
        }
        Ok(())
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
        Self::is_valid(&path, &CONFIG::config())?;
        Ok(Self {
            path,
            _m0: PhantomData,
        })
    }
}
