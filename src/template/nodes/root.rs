use crate::error::Result;
use crate::session::Session;
use async_recursion::async_recursion;
use config::{Config, ConfigEnum};
use config_derive::Config;
use druid::im::Vector;
use druid::{Data, ExtEventSink, Widget, WidgetExt, WidgetId};
use sha1::Digest;
use std::path::PathBuf;

use futures::future::{try_join_all, join_all};

use crate::settings::DownloadSettings;
use futures::prelude::*;
use serde::Serialize;
use std::sync::Arc;

use crate::template::communication::{Communication, RawCommunication};
use crate::template::node_type::{NodeType, NodeTypeData};
use crate::template::nodes::node::{Node, RawNode, Status};
use crate::template::nodes::root_data::RootNodeData;
use std::collections::HashSet;
use crate::widgets::tree::NodeIndex;
use crate::template::nodes::root_edit_data::RootNodeEditData;

#[derive(Config, Serialize, Debug)]
pub struct RawRootNode {
    #[config(ty = "Vec<Struct>")]
    pub children: Vec<RawNode>,
}


impl RawRootNode {
    pub fn transform(self, comm: RawCommunication) -> RootNode {
        RootNode {
            children: self
                .children
                .into_iter()
                .enumerate()
                .map(|(idx, raw_node)| raw_node.transform(vec![idx], comm.clone()))
                .collect(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct RootNode {
    pub children: Vec<Node>,
}

impl RootNode {
    pub fn raw(self) -> RawRootNode {
        RawRootNode {
            children: self
                .children
                .into_iter()
                .map(|node| node.raw())
                .collect(),
        }
    }

    pub async fn prepare(
        &mut self,
        session: &Session,
        dsettings: Arc<DownloadSettings>,
    ) -> Status {
        let futures: Vec<_> = self
            .children
            .iter_mut()
            .enumerate()
            .map(|(idx, child)| child.prepare(session, Arc::clone(&dsettings), PathBuf::new()))
            .collect();

        if join_all(futures).await.iter().any(|r| r == &Status::Failure) {
            Status::Failure
        } else {
            Status::Success
        }
    }

    pub async fn run(
        &self,
        session: &Session,
        dsettings: Arc<DownloadSettings>,
        indexes: Option<&HashSet<NodeIndex>>,
    ) {
        let futures: Vec<_> = self
            .children
            .iter()
            .map(|child| child.run(session, Arc::clone(&dsettings), indexes))
            .collect();

        join_all(futures).await;
    }

    pub fn widget_data(&self) -> RootNodeData {
        let children: Vector<_> = self
            .children
            .iter()
            .map(|node| node.widget_data())
            .collect();

        RootNodeData { children, selected: Vector::new() }
    }

    pub fn widget_edit_data(&self) -> RootNodeEditData {
        let children: Vector<_> = self
            .children
            .iter()
            .map(|node| node.widget_edit_data())
            .collect();

        RootNodeEditData { children, selected: Vector::new() }
    }
}
