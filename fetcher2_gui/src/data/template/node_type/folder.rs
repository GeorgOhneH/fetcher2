use druid::Data;
use fetcher2::template::node_type::Folder;

#[derive(Clone, Data, Debug)]
pub struct FolderData {
    name: String,
}

impl FolderData {
    pub fn new(folder: &Folder) -> Self {
        Self {
            name: folder.name.clone(),
        }
    }

    pub fn name(&self) -> String {
        self.name.clone()
    }
}
