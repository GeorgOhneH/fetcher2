use std::path::PathBuf;
use std::sync::Arc;

use config::{Config, ConfigEnum};
use druid::{Data, Lens};
use druid::im::Vector;

use crate::data::edit::EditWindowData;
use crate::data::settings::{OptionSettings, Settings};
use crate::data::template_info::TemplateInfoSelect;
use crate::data::win::{SubWindowInfo, WindowState};
use crate::template::node_type::NodeTypeData;
use crate::template::node_type::site::TaskMsg;
use crate::template::nodes::node_data::NodeData;
use crate::template::widget_data::TemplateData;

pub mod win;
pub mod settings;
pub mod template_info;
pub mod edit;

#[derive(Clone, Lens, Debug, Data, Config)]
pub struct AppData {
    #[config(ty = "struct")]
    pub template: TemplateData,

    #[config(ty = "Vec<_>")]
    pub recent_templates: Vector<Arc<PathBuf>>,

    #[config(ty = "struct")]
    pub settings_window: SubWindowInfo<OptionSettings>,

    #[config(ty = "enum", default = "Nothing")]
    pub template_info_select: TemplateInfoSelect,

    #[data(ignore)]
    #[config(ty = "_<struct>")]
    pub main_window: Option<WindowState>,

    #[config(ty = "struct")]
    pub edit_window: SubWindowInfo<EditWindowData>,

    #[config(default = 0.5)]
    pub split_point: f64,

    #[data(ignore)]
    pub folder_header_sizes: Vec<f64>,
}


impl AppData {
    pub fn get_settings(&self) -> Option<&Settings> {
        self.settings_window.data.settings.as_ref()
    }
    pub fn get_selected_node(&self) -> Option<&NodeData> {
        if self.template.root.selected.len() > 0 {
            let data_idx = &self.template.root.selected[0];
            let idx = data_idx.clone().into_iter().collect::<Vec<_>>();
            Some(self.template.node(&idx))
        } else {
            None
        }
    }

    pub fn get_selected_history(&self) -> Option<Vector<TaskMsg>> {
        self.get_selected_node()
            .map(|node| match &node.ty {
                NodeTypeData::Folder(_) => None,
                NodeTypeData::Site(site) => Some(site.history.clone()),
            })
            .flatten()
    }
}

