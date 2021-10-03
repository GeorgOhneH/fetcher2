use std::path::PathBuf;

use config::Config;

use crate::error::Result;

#[derive(Config, Debug, Clone, PartialEq)]
pub struct Folder {
    pub name: String,
}

impl Folder {
    pub async fn path_segment(&self) -> Result<PathBuf> {
        Ok(PathBuf::from(&self.name))
    }
}
