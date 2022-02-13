use config::traveller::Travel;
use druid::Data;
use fetcher2::template::node_type::NodeType;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use crate::data::template_edit::node_type::folder::FolderEditData;
use crate::data::template_edit::node_type::site_edit::SiteEditData;

pub mod folder;
pub mod site_edit;

#[derive(Debug, Clone, Data, Serialize, Deserialize, Travel)]
pub struct NodeTypeEditData {
    pub kind: NodeTypeEditKindData,
}

impl NodeTypeEditData {
    pub fn new(ty: &NodeType) -> Self {
        let kind = match ty {
            NodeType::Site(site) => NodeTypeEditKindData::Site(SiteEditData::new(&(*site))),
            NodeType::Folder(folder) => NodeTypeEditKindData::Folder(FolderEditData::new(folder)),
        };
        NodeTypeEditData { kind }
    }
    pub fn invalidate_cache(&mut self) {
        self.kind.invalidate_cache()
    }
}

#[derive(Debug, Clone, Data, Serialize, Deserialize, Travel)]
pub enum NodeTypeEditKindData {
    Folder(FolderEditData),
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
