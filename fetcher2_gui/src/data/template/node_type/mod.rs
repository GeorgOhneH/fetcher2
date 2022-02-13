use druid::Data;

use fetcher2::template::node_type::{NodeType, Site};

use crate::data::template::node_type::folder::FolderData;
use crate::data::template::node_type::site::SiteData;
use crate::data::template::node_type::site_state::SiteState;

pub mod folder;
pub mod site;
pub mod site_state;

#[derive(Debug, Clone, Data)]
pub enum NodeTypeData {
    Folder(FolderData),
    Site(SiteData),
}

impl NodeTypeData {
    pub fn new(ty: &NodeType) -> Self {
        match ty {
            NodeType::Site(site) => NodeTypeData::Site(SiteData::new(&(*site))),
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
