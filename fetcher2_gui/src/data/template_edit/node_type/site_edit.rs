use config::traveller::Travel;
use druid::Data;
use fetcher2::site_modules::Module;
use fetcher2::template::node_type::{Site, SiteStorage};
use fetcher2::template::DownloadArgs;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Debug, Clone, Data, Serialize, Deserialize, Travel)]
pub struct SiteEditData {
    pub module: Module,

    pub download_args: Option<DownloadArgs>,

    #[data(ignore)]
    #[travel(skip)]
    pub storage: Option<Arc<SiteStorage>>,
}

impl SiteEditData {
    pub fn new(site: &Site) -> Self {
        Self {
            module: site.module.clone(),
            download_args: site.download_args.clone(),
            storage: Some(site.storage.clone()),
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
