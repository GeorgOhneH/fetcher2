use std::path::PathBuf;
use std::sync::Arc;

use druid::{Data, Lens};
use druid::im::Vector;

use config::{Config, ConfigEnum};
use fetcher2::template::node_type::site::TaskMsg;
use template::node_type::NodeTypeData;
use template::nodes::node::NodeData;

use crate::data::edit::EditWindowData;
use crate::data::settings::{OptionSettings, Settings};
use crate::data::template_info::{TemplateInfo, TemplateInfoSelect};
use crate::data::win::{SubWindowInfo, WindowState};
use crate::data::template::TemplateData;

pub mod edit;
pub mod settings;
pub mod template_info;
pub mod win;
pub mod template;
pub mod template_edit;

#[derive(Clone, Lens, Debug, Data, Config)]
pub struct AppData {
    #[config(ty = "struct")]
    pub template: TemplateData,

    #[config(ty = "Vec<_>")]
    pub recent_templates: Vector<Arc<PathBuf>>,

    #[config(ty = "struct")]
    pub settings_window: SubWindowInfo<OptionSettings>,

    #[config(ty = "struct")]
    pub template_info: TemplateInfo,

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
        if !self.template.root.selected.is_empty() {
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
