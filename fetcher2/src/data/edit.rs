use config::Config;
use druid::{Data, Lens};

use crate::data::win::WindowState;
use crate::template::widget_edit_data::TemplateEditData;

#[derive(Config, Debug, Data, Clone, Lens)]
pub struct EditWindowData {
    #[config(ty = "_<struct>")]
    pub node_win_state: Option<WindowState>,

    #[config(ty = "struct")]
    pub edit_template: TemplateEditData,
}
