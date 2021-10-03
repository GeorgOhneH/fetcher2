use druid::Data;

use config::Config;
use fetcher2::template::node_type::Folder;

#[derive(Clone, Data, Debug, Config)]
pub struct FolderEditData {
    name: String,
}

impl FolderEditData {
    pub fn new(folder: Folder) -> Self {
        Self { name: folder.name }
    }
    pub fn raw(self) -> Folder {
        Folder { name: self.name }
    }
    pub fn name(&self) -> String {
        self.name.clone()
    }
}
