use std::sync::Arc;

use config::Config;
use config::ConfigEnum;
use druid::Data;

use fetcher2::site_modules::Module;
use fetcher2::template::DownloadArgs;
use fetcher2::template::node_type::{SiteStorage, Site};

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
    pub fn new(site: Site) -> Self {
        Self {
            module: site.module,
            download_args: site.download_args,
            storage: Some(site.storage),
        }
    }
    pub fn raw(self) -> Site {
        Site {
            module: self.module,
            storage: self.storage.unwrap_or_else(|| Arc::new(SiteStorage::new())),
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
