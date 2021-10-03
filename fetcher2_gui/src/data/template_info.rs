use druid::Data;

use config::ConfigEnum;

#[derive(Clone, Copy, Debug, Data, PartialEq, ConfigEnum)]
pub enum TemplateInfoSelect {
    Nothing,
    General,
    Folder,
    History,
}
