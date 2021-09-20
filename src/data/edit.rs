use crate::template::widget_edit_data::TemplateEditData;
use crate::data::win::WindowState;

use druid::{Lens, Data};
use config::Config;

#[derive(Config, Debug, Data, Clone, Lens)]
pub struct EditWindowData {
    #[config(ty = "_<struct>")]
    pub node_win_state: Option<WindowState>,

    #[config(skip = TemplateEditData::new())]
    pub edit_template: TemplateEditData,
}
