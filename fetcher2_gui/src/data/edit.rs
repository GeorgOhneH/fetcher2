use config::Config;
use druid::{Data, Lens};

use crate::data::template_edit::TemplateEditData;
use crate::data::win::WindowState;

#[derive(Config, Debug, Data, Clone, Lens)]
pub struct EditWindowData {
    #[config(ty = "_<struct>")]
    pub node_win_state: Option<WindowState>,

    #[config(ty = "struct")]
    pub edit_template: TemplateEditData,
}
