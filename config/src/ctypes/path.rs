use crate::*;
use druid::im;
use druid::widget::{Flex, Label, TextBox, Maybe};
use druid::{Data, Lens, LensExt, Widget, WidgetExt};
use std::path::PathBuf;

#[derive(Debug, Clone, Data)]
pub enum CPathTypes {
    Folder,
    File(Option<im::Vector<String>>),
    Any,
}

#[derive(Debug, Clone, Data, Lens)]
pub struct CPath {
    #[data(same_fn = "PartialEq::eq")]
    value: Option<PathBuf>,
    ty: CPathTypes,
    #[data(ignore)]
    name: Option<String>,
}

impl CPath {
    fn new() -> Self {
        Self {
            value: None,
            ty: CPathTypes::Any,
            name: None,
        }
    }

    pub fn is_valid(&self, path: &PathBuf) -> Result<(), InvalidError> {
        if path.is_relative() {
            return Err(InvalidError::new("Path must be absolute"));
        }
        if !path.exists() {
            return Err(InvalidError::new("Path must exist"));
        }
        match &self.ty {
            CPathTypes::Any => Ok(()),
            CPathTypes::Folder => {
                if path.is_dir() {
                    Ok(())
                } else {
                    Err(InvalidError::new("Path must be a folder"))
                }
            }
            CPathTypes::File(extensions) => {
                if !path.is_file() {
                    return Err(InvalidError::new("Path must be a file"));
                }
                if let Some(extens) = extensions {
                    if let Some(file_extension) = path.extension() {
                        let ex = file_extension
                            .to_str()
                            .ok_or(InvalidError::new("Path must have a extension"))?;
                        if extens.iter().any(|ext| ext == ex) {
                            Ok(())
                        } else {
                            Err(InvalidError::new("Path must have a extension"))
                        }
                    } else {
                        Err(InvalidError::new("Path must have a extension"))
                    }
                } else {
                    Ok(())
                }
            }
        }
    }

    pub fn get(&self) -> Option<&PathBuf> {
        Option::from(&self.value)
    }

    pub fn set<T>(&mut self, path: T) -> Result<(), InvalidError>
    where
        PathBuf: From<T>,
    {
        let path = PathBuf::from(path);
        self.is_valid(&path)?;
        self.value = Some(path);
        Ok(())
    }

    pub fn unset(&mut self) {
        self.value = None;
    }

    pub fn state(&self) -> State {
        match &self.value {
            Some(v) => self.is_valid(v).into(),
            None => State::None,
        }
    }

    pub fn widget() -> impl Widget<Self> {
        Flex::row()
            .with_child(Maybe::or_empty(|| Label::dynamic(|data: &String, _| data.clone() + ":")).lens(Self::name))
            .with_child(TextBox::new().lens(Self::value.map(
                |value| {
                    match value {
                        Some(v) => v
                            .clone()
                            .into_os_string()
                            .into_string()
                            .unwrap_or("".to_owned()),
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
    }
}

pub struct CPathBuilder {
    inner: CPath,
}

impl CPathBuilder {
    pub fn new() -> Self {
        Self {
            inner: CPath::new(),
        }
    }
    pub fn default(mut self, value: String) -> Self {
        self.inner.set(value).unwrap();
        self
    }

    pub fn name(mut self, name: String) -> Self {
        self.inner.name = Some(name);
        self
    }

    pub fn path_ty(mut self, ty: CPathTypes) -> Self {
        self.inner.ty = ty;
        self
    }
    pub fn build(self) -> CPath {
        self.inner
    }
}
