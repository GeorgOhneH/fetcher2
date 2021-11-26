use druid::{Data, Lens};

use config::{ConfigEnum, Config};

#[derive(Debug, Clone, Data, Lens, Config)]
pub struct TemplateInfo {
    #[config(ty = "struct")]
    #[data(ignore)]
    pub folder: FolderInfo,

    #[config(ty = "struct")]
    #[data(ignore)]
    pub history: HistoryInfo,

    #[config(ty = "enum", default = "Nothing")]
    pub selected: TemplateInfoSelect
}

#[derive(Debug, Clone, Lens, Config)]
pub struct FolderInfo {
    pub header_sizes: Vec<f64>,
}

#[derive(Debug, Clone, Lens, Config)]
pub struct HistoryInfo {
    pub header_sizes: Vec<f64>,
}

#[derive(Clone, Copy, Debug, Data, PartialEq, ConfigEnum)]
pub enum TemplateInfoSelect {
    Nothing,
    General,
    Folder,
    History,
}
