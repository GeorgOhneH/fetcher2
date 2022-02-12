use config::Config;
use druid::{Data, Lens};
use druid::im::Vector;
use nodes::root::RootNodeEditData;
use std::fmt::Debug;
use std::path::PathBuf;

pub mod nodes;
pub mod node_type;

#[derive(Debug, Clone, Data, Lens, Config)]
pub struct TemplateEditData {
    #[config(skip = RootNodeEditData::empty())]
    pub root: RootNodeEditData,

    #[data(eq)]
    #[config(skip = None)]
    pub save_path: Option<PathBuf>,

    #[config(ty = "Vec<_>")]
    pub header_sizes: Vector<f64>,
}

impl TemplateEditData {
    pub fn reset(&mut self) {
        self.save_path = None;
        self.root = RootNodeEditData::empty();
    }
}
