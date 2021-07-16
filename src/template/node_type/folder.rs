use crate::error::{Result};
use std::path::PathBuf;
use serde::Serialize;
use config_derive::Config;
use config::Config;
use druid::Data;

#[derive(Config, Serialize, Debug)]
pub struct Folder {
    name: String,
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
}

#[derive(Clone, Data, Debug)]
pub struct FolderData {
    name: String,
}