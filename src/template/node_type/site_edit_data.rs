use std::sync::Arc;

use config::Config;
use config::ConfigEnum;
use druid::Data;

use crate::site_modules::Module;
use crate::template::DownloadArgs;
use crate::template::node_type::{Site, SiteStorage};

#[derive(Debug, Clone, Data, Config)]
pub struct SiteEditData {
    #[config(ty = "enum")]
    pub module: Module,

    #[config(ty = "_<struct>")]
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
