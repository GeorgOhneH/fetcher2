use std::path::PathBuf;
use serde::Serialize;
use serde::Deserialize;
use config::traveller::Travel;

use crate::error::Result;

#[derive(Travel, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Folder {
    pub name: String,
}

impl Folder {
    pub async fn path_segment(&self) -> Result<PathBuf> {
        Ok(PathBuf::from(&self.name))
    }
}
