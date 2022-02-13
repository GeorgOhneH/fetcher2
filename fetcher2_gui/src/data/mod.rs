use std::path::PathBuf;
use std::sync::Arc;

use druid::im::Vector;
use druid::{Data, Lens};
use serde::{Deserialize, Serialize};

use fetcher2::template::node_type::site::TaskMsg;
use template::node_type::NodeTypeData;
use template::nodes::node::NodeData;

use crate::data::edit::EditWindowData;
use crate::data::settings::{OptionSettings, Settings};
use crate::data::template::TemplateData;
use crate::data::template_info::TemplateInfo;
use crate::data::win::{SubWindowInfo, WindowState};

pub mod edit;
pub mod settings;
pub mod template;
pub mod template_edit;
pub mod template_info;
pub mod win;

#[derive(Clone, Lens, Debug, Data, Serialize, Deserialize, Default)]
pub struct AppData {
    pub template: TemplateData,

    pub recent_templates: Vector<Arc<PathBuf>>,

    pub settings_window: SubWindowInfo<OptionSettings>,

    pub template_info: TemplateInfo,

    #[data(ignore)]
    pub main_window: Option<WindowState>,

    pub edit_window: SubWindowInfo<EditWindowData>,

    pub split_point: Option<f64>,

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
            Some(self.template.node(data_idx))
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
