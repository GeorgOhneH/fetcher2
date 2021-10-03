use std::path::PathBuf;
use std::sync::Arc;

use config::{Config, ConfigEnum};
use druid::Data;
use serde::Deserialize;
use serde::Serialize;

use crate::error::Result;
use crate::session::Session;
pub use crate::template::node_type::folder::Folder;
pub use crate::template::node_type::site::Mode;
pub use crate::template::node_type::site::Site;
pub use crate::template::node_type::site::SiteStorage;
pub use crate::template::node_type::site::{DownloadArgs, Extensions};
use crate::settings::DownloadSettings;

pub mod folder;
pub mod site;
mod utils;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum NodeType {
    Folder(Folder),
    Site(Arc<Site>),
}

impl NodeType {
    pub async fn path_segment(
        &self,
        session: &Session,
        dsettings: &DownloadSettings,
    ) -> Result<PathBuf> {
        match self {
            NodeType::Folder(folder) => folder.path_segment().await,
            NodeType::Site(site) => site.path_segment(session, dsettings).await,
        }
    }
}
