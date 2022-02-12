use std::path::PathBuf;

use serde::Deserialize;
use serde::Serialize;

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
