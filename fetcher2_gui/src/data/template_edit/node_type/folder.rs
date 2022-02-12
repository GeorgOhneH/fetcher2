use druid::Data;
use serde::{Deserialize, Serialize};
use config::traveller::Travel;
use fetcher2::template::node_type::Folder;

#[derive(Clone, Data, Debug, Serialize, Deserialize, Travel)]
pub struct FolderEditData {
    name: String,
}

impl FolderEditData {
    pub fn new(folder: &Folder) -> Self {
        Self { name: folder.name.clone() }
    }
    pub fn raw(self) -> Folder {
        Folder { name: self.name }
    }
    pub fn name(&self) -> String {
        self.name.clone()
    }
}
