use druid::im::Vector;
use druid::{Data, Lens};
use nodes::root::RootNodeEditData;
use serde::{Deserialize, Serialize};
use std::fmt::Debug;
use std::path::PathBuf;

pub mod node_type;
pub mod nodes;

#[derive(Debug, Clone, Data, Lens, Serialize, Deserialize, Default)]
pub struct TemplateEditData {
    #[serde(skip)]
    pub root: RootNodeEditData,

    #[data(eq)]
    #[serde(skip)]
    pub save_path: Option<PathBuf>,

    pub header_sizes: Vector<f64>,
}

impl TemplateEditData {
    pub fn reset(&mut self) {
        self.save_path = None;
        self.root = RootNodeEditData::empty();
    }
}
