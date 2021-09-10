use crate::site_modules::Module;
use crate::template::DownloadArgs;
use druid::Data;
use crate::template::node_type::SiteStorage;
use std::sync::Arc;


#[derive(Debug, Clone, Data)]
pub struct SiteEditData {
    pub module: Module,

    pub download_args: Option<DownloadArgs>,

    #[data(ignore)]
    pub storage: Option<Arc<SiteStorage>>,
}


impl SiteEditData {
    pub fn name(&self) -> String {
        self.module.name()
    }
}
