use crate::*;
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub enum CPathTypes {
    Folder,
    File(Option<Vec<String>>),
    Any,
}

#[derive(Debug, Clone)]
pub struct CPath {
    value: Option<PathBuf>,
    ty: CPathTypes,
}

impl CPath {
    fn new() -> Self {
        Self {
            value: None,
            ty: CPathTypes::Any,
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
    pub fn default(mut self, value: &str) -> Self {
        self.inner.set(value).unwrap();
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