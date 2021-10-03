pub mod nodes;
pub mod node_type;

use std::fmt::Debug;
use std::path::PathBuf;

use druid::im::Vector;
use druid::{Data, Lens};

use config::Config;

use nodes::root::RootNodeEditData;

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
