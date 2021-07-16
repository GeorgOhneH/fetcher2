use crate::error::{Result};
use crate::session::Session;
use config::{Config, ConfigEnum};
use config_derive::Config;
use serde::Serialize;

mod folder;
mod site;
mod utils;

use crate::settings::DownloadSettings;
pub use crate::template::node_type::folder::Folder;
pub use crate::template::node_type::site::Mode;
pub use crate::template::node_type::site::Site;
pub use crate::template::node_type::site::SiteStorage;
pub use crate::template::node_type::site::{DownloadArgs, Extensions};
use std::sync::Arc;
use druid::Data;
use std::path::PathBuf;
use crate::template::node_type::site::SiteData;
use crate::template::node_type::folder::FolderData;

#[derive(Config, Serialize, Debug)]
pub enum NodeType {
    #[config(ty = "struct")]
    Folder(Folder),
    #[config(inner_ty = "struct")]
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

    pub fn widget_data(&self) -> NodeTypeData {
        match self {
            NodeType::Site(site) => NodeTypeData::Site(site.widget_data()),
            NodeType::Folder(folder) => NodeTypeData::Folder(folder.widget_data()),
        }
    }
}

#[derive(Debug, Clone, Data)]
pub enum NodeTypeData {
    Folder(FolderData),
    Site(SiteData),
}