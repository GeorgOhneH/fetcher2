use druid::{Data, Lens};
use serde::{Deserialize, Serialize};

use crate::data::template_edit::TemplateEditData;
use crate::data::win::WindowState;

#[derive(Serialize, Deserialize, Debug, Data, Clone, Lens, Default)]
pub struct EditWindowData {
    pub node_win_state: Option<WindowState>,

    pub edit_template: TemplateEditData,
}
