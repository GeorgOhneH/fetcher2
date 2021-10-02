use config::ConfigEnum;
use druid::{Data, Lens};

#[derive(Clone, Copy, Debug, Data, PartialEq, ConfigEnum)]
pub enum TemplateInfoSelect {
    Nothing,
    General,
    Folder,
    History,
}
