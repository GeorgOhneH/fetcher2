use druid::{Data, Lens};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Data, Lens, Serialize, Deserialize, Default)]
pub struct TemplateInfo {
    #[data(ignore)]
    pub folder: FolderInfo,

    #[data(ignore)]
    pub history: HistoryInfo,

    #[serde(default)]
    pub selected: TemplateInfoSelect
}

#[derive(Debug, Clone, Lens, Serialize, Deserialize, Default)]
pub struct FolderInfo {
    pub header_sizes: Vec<f64>,
}

#[derive(Debug, Clone, Lens, Serialize, Deserialize, Default)]
pub struct HistoryInfo {
    pub header_sizes: Vec<f64>,
}

#[derive(Clone, Copy, Debug, Data, PartialEq, Serialize, Deserialize)]
pub enum TemplateInfoSelect {
    Nothing,
    General,
    Folder,
    History,
}

impl Default for TemplateInfoSelect {
    fn default() -> Self {
        Self::Nothing
    }
}
