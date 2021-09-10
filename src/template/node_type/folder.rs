use crate::error::{Result};
use std::path::PathBuf;
use serde::Serialize;
use config_derive::Config;
use config::Config;
use druid::Data;

#[derive(Config, Serialize, Debug, Clone)]
pub struct Folder {
    name: String,
}


impl From<FolderEditData> for Folder {
    fn from(data: FolderEditData) -> Self {
        Self {
            name: data.name
        }
    }
}

impl Folder {
    pub async fn path_segment(&self) -> Result<PathBuf> {
        Ok(PathBuf::from(&self.name))
    }

    pub fn widget_data(&self) -> FolderData {
        FolderData {
            name: self.name.clone()
        }
    }

    pub fn widget_edit_data(&self) -> FolderEditData {
        FolderEditData {
            name: self.name.clone()
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


#[derive(Clone, Data, Debug)]
pub struct FolderEditData {
    name: String,
}

impl FolderEditData {
    pub fn name(&self) -> String {
        self.name.clone()
    }
}