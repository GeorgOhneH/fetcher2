use std::path::PathBuf;
use std::sync::Arc;

use config::{Config, ConfigEnum};
use druid::Data;
use serde::Deserialize;
use serde::Serialize;

use crate::template::node_type::folder::{FolderData, FolderEditData};
use crate::template::node_type::site_data::{SiteData, SiteState};
use crate::template::node_type::site_edit_data::SiteEditData;
use fetcher2::template::node_type::{Site, NodeType};

pub mod folder;
pub mod site_data;
pub mod site_edit_data;

#[derive(Debug, Clone, Data)]
pub enum NodeTypeData {
    Folder(FolderData),
    Site(SiteData),
}

impl NodeTypeData {
    pub fn new(ty: NodeType) -> Self {
        match ty {
            NodeType::Site(site) => NodeTypeData::Site(SiteData::new((*site).clone())),
            NodeType::Folder(folder) => NodeTypeData::Folder(FolderData::new(folder)),
        }
    }
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
    #[config(ty = "enum")]
    pub kind: NodeTypeEditKindData,
}

impl NodeTypeEditData {
    pub fn new(ty: NodeType) -> Self {
        let kind = match ty {
            NodeType::Site(site) => NodeTypeEditKindData::Site(SiteEditData::new((*site).clone())),
            NodeType::Folder(folder) => NodeTypeEditKindData::Folder(FolderEditData::new(folder)),
        };
        NodeTypeEditData { kind }
    }
    pub fn invalidate_cache(&mut self) {
        self.kind.invalidate_cache()
    }
}

#[derive(Debug, Clone, Data, ConfigEnum)]
pub enum NodeTypeEditKindData {
    #[config(ty = "struct")]
    Folder(FolderEditData),
    #[config(ty = "struct")]
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
