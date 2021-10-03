use druid::{Data, Lens};

use config::Config;

use crate::data::win::WindowState;
use crate::data::template_edit::TemplateEditData;

#[derive(Config, Debug, Data, Clone, Lens)]
pub struct EditWindowData {
    #[config(ty = "_<struct>")]
    pub node_win_state: Option<WindowState>,

    #[config(ty = "struct")]
    pub edit_template: TemplateEditData,
}
