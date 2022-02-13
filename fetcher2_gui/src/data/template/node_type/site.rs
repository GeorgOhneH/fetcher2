use druid::im::Vector;
use druid::Data;

use fetcher2::site_modules::Module;
use fetcher2::template::node_type::site::TaskMsg;
use fetcher2::template::DownloadArgs;

use crate::data::template::node_type::site_state::SiteState;
use crate::data::template::node_type::Site;

#[derive(Debug, Clone, Data)]
pub struct SiteData {
    pub module: Module,

    pub download_args: Option<DownloadArgs>,

    pub history: Vector<TaskMsg>,

    pub state: SiteState,
}

impl SiteData {
    pub fn new(site: &Site) -> Self {
        Self {
            module: site.module.clone(),
            download_args: site.download_args.clone(),
            history: site.storage.history.lock().unwrap().clone().into(),
            state: SiteState::new(),
        }
    }

    pub fn name(&self) -> String {
        self.module.name()
    }

    pub fn added_replaced(&self) -> (usize, usize) {
        (
            self.state.download.new_added,
            self.state.download.new_replaced,
        )
    }
}
