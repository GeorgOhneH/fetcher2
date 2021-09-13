use crate::site_modules::Module;
use crate::template::DownloadArgs;
use druid::Data;
use crate::template::node_type::SiteStorage;
use std::sync::Arc;
use config::Config;
use config_derive::Config;
use config::ConfigEnum;

#[derive(Debug, Clone, Data, Config)]
pub struct SiteEditData {
    #[config(ty = "Enum")]
    pub module: Module,

    #[config(ty = "_<Struct>")]
    pub download_args: Option<DownloadArgs>,

    #[data(ignore)]
    #[config(skip = None)]
    pub storage: Option<Arc<SiteStorage>>,
}


impl SiteEditData {
    pub fn name(&self) -> String {
        self.module.name()
    }

    pub fn invalidate_cache(&mut self) {
        self.storage = None
    }
}
