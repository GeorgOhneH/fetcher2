use std::fmt::Formatter;
use std::marker::PhantomData;
use std::ops::{Deref, DerefMut};
use std::path::{Path, PathBuf};

use crate::errors::Error;
use crate::traveller::{Travel, TravelPathConfig, Traveller};
use druid::im::Vector;
use druid::lens::Field;
use druid::widget::{Button, Flex, Label, Maybe, TextBox};
use druid::{Data, Lens, LensExt, Widget, WidgetExt};
use druid::{FileDialogOptions, FileSpec};
use druid_widget_nursery::WidgetExt as _;
use serde::de::{EnumAccess, MapAccess, SeqAccess, Visitor};
use serde::{Deserialize, Deserializer, Serialize, Serializer};

#[derive(Debug, Clone, Data, Lens)]
pub struct CPath {
    #[data(same_fn = "PartialEq::eq")]
    pub value: Option<PathBuf>,
    #[data(ignore)]
    path_config: TravelPathConfig,
    #[data(ignore)]
    name: Option<&'static str>,
}

impl CPath {
    pub fn new(path_config: TravelPathConfig) -> Self {
        Self {
            value: None,
            path_config,
            name: None,
        }
    }

    pub fn is_valid(&self, path: &Path) -> Result<(), Error> {
        todo!()
    }

    pub fn widget() -> impl Widget<Self> {
        Flex::row()
            .with_child(
                Maybe::or_empty(|| Label::dynamic(|data: &&'static str, _| format!("{data}:")))
                    .lens(Self::name),
            )
            .with_child(TextBox::new().lens(Self::value.map(
                |value| {
                    match value {
                        Some(v) => v
                            .clone()
                            .into_os_string()
                            .into_string()
                            .unwrap_or_else(|_| "".to_owned()),
                        None => "".to_owned(),
                    }
                },
                |value: &mut Option<PathBuf>, x| {
                    if x.is_empty() {
                        *value = None
                    } else {
                        *value = Some(PathBuf::from(x))
                    }
                },
            )))
            .with_child(
                Button::new("+")
                    .on_click(|ctx, data: &mut Self, _env| {
                        let open_dialog_options = FileDialogOptions::new()
                            .select_directories()
                            .name_label("Target")
                            .title("Choose a target for this lovely file")
                            .button_text("Export")
                            .default_name("MySavedFile.txt");
                        let open_dialog_options = match &data.path_config {
                            TravelPathConfig::AbsoluteExistDir => {
                                open_dialog_options.select_directories()
                            }
                            TravelPathConfig::AbsoluteExistFile(allowed) => open_dialog_options
                                .allowed_types(allowed.clone().into_iter().collect()),
                            _ => open_dialog_options,
                        };
                        ctx.submit_command(
                            druid::commands::SHOW_OPEN_PANEL.with(open_dialog_options),
                        )
                    })
                    .on_command(
                        druid::commands::OPEN_FILE,
                        |ctx, file_info, data: &mut Self| {
                            // TODO doesn't work for multiple
                            data.value = Some(file_info.path.clone());
                            ctx.request_update();
                        },
                    ),
            )
    }
}

pub trait PathConfig {
    fn config() -> TravelPathConfig;
}

#[derive(Debug)]
pub struct AnyPath;
#[derive(Debug)]
pub struct Relative;
#[derive(Debug)]
pub struct Absolute;
#[derive(Debug)]
pub struct AbsoluteExist;
#[derive(Debug)]
pub struct AbsoluteExistFile;
#[derive(Debug)]
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

#[derive(Debug)]
pub struct StrictPath<CONFIG>
where
    CONFIG: PathConfig,
{
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
