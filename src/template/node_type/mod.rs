use crate::error::Result;
use crate::session::Session;
use config::{Config, ConfigEnum};
use serde::Serialize;

pub mod folder;
pub mod site;
pub mod site_data;
pub mod site_edit_data;
mod utils;

use crate::settings::DownloadSettings;
pub use crate::template::node_type::folder::Folder;
use crate::template::node_type::folder::{FolderData, FolderEditData};
pub use crate::template::node_type::site::Mode;
pub use crate::template::node_type::site::Site;
pub use crate::template::node_type::site::SiteStorage;
pub use crate::template::node_type::site::{DownloadArgs, Extensions};
use crate::template::node_type::site_data::{SiteData, SiteState};
use crate::template::node_type::site_edit_data::SiteEditData;
use crate::template::nodes::node::MetaData;
use druid::Data;
use std::path::PathBuf;
use std::sync::Arc;

#[derive(ConfigEnum, Serialize, Debug, Clone)]
pub enum NodeType {
    #[config(ty = "Struct")]
    Folder(Folder),
    #[config(ty = "_<Struct>")]
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

    pub fn widget_edit_data(&self, meta_data: MetaData) -> NodeTypeEditData {
        let kind = match self {
            NodeType::Site(site) => NodeTypeEditKindData::Site(site.widget_edit_data()),
            NodeType::Folder(folder) => NodeTypeEditKindData::Folder(folder.widget_edit_data()),
        };
        NodeTypeEditData { kind, meta_data }
    }
}

#[derive(Debug, Clone, Data)]
pub enum NodeTypeData {
    Folder(FolderData),
    Site(SiteData),
}

impl NodeTypeData {
    pub fn folder(&self) -> Option<&FolderData> {
        match self {
            NodeTypeData::Folder(folder) => Some(folder),
            _ => None,
        }
    }
    pub fn folder_mut(&mut self) -> Option<&mut FolderData> {
        match self {
            NodeTypeData::Folder(folder) => Some(folder),
            _ => None,
        }
    }
    pub fn site(&self) -> Option<&SiteData> {
        match self {
            NodeTypeData::Site(site) => Some(site),
            _ => None,
        }
    }
    pub fn site_mut(&mut self) -> Option<&mut SiteData> {
        match self {
            NodeTypeData::Site(site) => Some(site),
            _ => None,
        }
    }

    pub fn name(&self) -> String {
        match self {
            Self::Folder(folder) => folder.name(),
            Self::Site(site) => site.name(),
        }
    }

    pub fn is_finished(&self) -> bool {
        match self {
            Self::Folder(_) => true,
            Self::Site(site) => site.state.run == 0,
        }
    }

    pub fn reset_state(&mut self) {
        match self {
            Self::Folder(_) => (),
            Self::Site(site) => site.state = SiteState::new(),
        }
    }
}

#[derive(Debug, Clone, Data, Config)]
pub struct NodeTypeEditData {
    #[config(ty = "Enum")]
    pub kind: NodeTypeEditKindData,

    #[config(ty = "Struct")]
    pub meta_data: MetaData,
}

impl NodeTypeEditData {
    pub fn invalidate_cache(&mut self) {
        self.kind.invalidate_cache()
    }
}

#[derive(Debug, Clone, Data, ConfigEnum)]
pub enum NodeTypeEditKindData {
    #[config(ty = "Struct")]
    Folder(FolderEditData),
    #[config(ty = "Struct")]
    Site(SiteEditData),
}

impl NodeTypeEditKindData {
    pub fn raw(self) -> NodeType {
        match self {
            Self::Folder(folder) => NodeType::Folder(folder.raw()),
            Self::Site(site) => NodeType::Site(Arc::new(site.raw())),
        }
    }
    pub fn name(&self) -> String {
        match self {
            Self::Folder(folder) => folder.name(),
            Self::Site(site) => site.name(),
        }
    }

    pub fn invalidate_cache(&mut self) {
        match self {
            Self::Site(site_data) => site_data.invalidate_cache(),
            Self::Folder(_) => (),
        }
    }
}
