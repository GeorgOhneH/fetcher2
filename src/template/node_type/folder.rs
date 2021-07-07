use crate::error::{Result};
use std::path::PathBuf;
use serde::Serialize;
use config_derive::Config;
use config::Config;

#[derive(Config, Serialize, Debug)]
pub struct Folder {
    name: String,
}

impl Folder {
    pub async fn path_segment(&self) -> Result<PathBuf> {
        Ok(PathBuf::from(&self.name))
    }
}
