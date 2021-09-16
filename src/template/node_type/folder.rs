use crate::error::Result;
use config::Config;
use druid::Data;
use serde::Serialize;
use std::path::PathBuf;

#[derive(Config, Serialize, Debug, Clone)]
pub struct Folder {
    name: String,
}

impl From<FolderEditData> for Folder {
    fn from(data: FolderEditData) -> Self {
        Self { name: data.name }
    }
}

impl Folder {
    pub async fn path_segment(&self) -> Result<PathBuf> {
        Ok(PathBuf::from(&self.name))
    }

    pub fn widget_data(&self) -> FolderData {
        FolderData {
            name: self.name.clone(),
        }
    }

    pub fn widget_edit_data(&self) -> FolderEditData {
        FolderEditData {
            name: self.name.clone(),
        }
    }
}

#[derive(Clone, Data, Debug)]
pub struct FolderData {
    name: String,
}

impl FolderData {
    pub fn name(&self) -> String {
        self.name.clone()
    }
}

#[derive(Clone, Data, Debug, Config)]
pub struct FolderEditData {
    name: String,
}

impl FolderEditData {
    pub fn raw(self) -> Folder {
        Folder { name: self.name }
    }
    pub fn name(&self) -> String {
        self.name.clone()
    }
}
