use crate::site_modules::Module;
use crate::template::node_type::{Site, SiteStorage};
use crate::template::DownloadArgs;
use config::Config;
use config::ConfigEnum;
use config_derive::Config;
use druid::Data;
use std::sync::Arc;

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
    pub fn raw(self) -> Site {
        Site {
            module: self.module,
            storage: self.storage.unwrap_or(Arc::new(SiteStorage::new())),
            download_args: self.download_args,
        }
    }
    pub fn name(&self) -> String {
        self.module.name()
    }

    pub fn invalidate_cache(&mut self) {
        self.storage = None
    }
}