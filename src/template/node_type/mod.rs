use crate::error::{Result};
use crate::session::Session;
use config::{Config, ConfigEnum};
use config_derive::Config;
use serde::Serialize;

pub mod folder;
pub mod site;
mod utils;
pub mod site_data;
pub mod site_edit_data;

use crate::settings::DownloadSettings;
pub use crate::template::node_type::folder::Folder;
pub use crate::template::node_type::site::Mode;
pub use crate::template::node_type::site::Site;
pub use crate::template::node_type::site::SiteStorage;
pub use crate::template::node_type::site::{DownloadArgs, Extensions};
use std::sync::Arc;
use druid::Data;
use std::path::PathBuf;
use crate::template::node_type::folder::{FolderData, FolderEditData};
use crate::template::node_type::site_data::SiteData;
use crate::template::node_type::site_edit_data::SiteEditData;

#[derive(Config, Serialize, Debug, Clone)]
pub enum NodeType {
    #[config(ty = "Struct")]
    Folder(Folder),
    #[config(ty = "_<Struct>")]
    Site(Arc<Site>),
}


impl From<NodeTypeEditData> for NodeType {
    fn from(data: NodeTypeEditData) -> Self {
        match data {
            NodeTypeEditData::Folder(folder) => Self::Folder(folder.into()),
            NodeTypeEditData::Site(site) => Self::Site(Arc::new(site.into())),
        }
    }
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

    pub fn widget_edit_data(&self) -> NodeTypeEditData {
        match self {
            NodeType::Site(site) => NodeTypeEditData::Site(site.widget_edit_data()),
            NodeType::Folder(folder) => NodeTypeEditData::Folder(folder.widget_edit_data()),
        }
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
}


#[derive(Debug, Clone, Data)]
pub enum NodeTypeEditData {
    Folder(FolderEditData),
    Site(SiteEditData),
}

impl NodeTypeEditData {
    pub fn name(&self) -> String {
        match self {
            Self::Folder(folder) => folder.name(),
            Self::Site(site) => site.name(),
        }
    }
}