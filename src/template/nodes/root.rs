use std::collections::HashSet;
use std::path::PathBuf;
use std::sync::Arc;

use async_recursion::async_recursion;
use config::{Config, ConfigEnum};
use druid::{Data, ExtEventSink, Widget, WidgetExt, WidgetId};
use druid::im::Vector;
use futures::future::{join_all, try_join_all};
use futures::prelude::*;
use serde::Deserialize;
use serde::Serialize;
use sha1::Digest;

use crate::data::settings::DownloadSettings;
use crate::error::Result;
use crate::session::Session;
use crate::template::communication::{Communication, RawCommunication};
use crate::template::node_type::{NodeType, NodeTypeData};
use crate::template::nodes::node::{Node, RawNode, Status};
use crate::template::nodes::root_data::RootNodeData;
use crate::template::nodes::root_edit_data::RootNodeEditData;
use crate::widgets::tree::NodeIndex;

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct RawRootNode {
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
    pub fn new() -> Self {
        Self {
            children: Vec::new(),
        }
    }
    pub fn raw(self) -> RawRootNode {
        RawRootNode {
            children: self.children.into_iter().map(|node| node.raw()).collect(),
        }
    }

    pub async fn prepare(&mut self, session: &Session, dsettings: Arc<DownloadSettings>) -> Status {
        let futures: Vec<_> = self
            .children
            .iter_mut()
            .enumerate()
            .map(|(idx, child)| child.prepare(session, Arc::clone(&dsettings), PathBuf::new()))
            .collect();

        if join_all(futures)
            .await
            .iter()
            .any(|r| r == &Status::Failure)
        {
            Status::Failure
        } else {
            Status::Success
        }
    }

    // indexes: None means all
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

    pub fn inform_of_cancel(&self) {
        for child in &self.children {
            child.inform_of_cancel()
        }
    }

    pub fn widget_data(&self) -> RootNodeData {
        let children: Vector<_> = self
            .children
            .iter()
            .map(|node| node.widget_data())
            .collect();

        RootNodeData {
            children,
            selected: Vector::new(),
        }
    }

    pub fn widget_edit_data(&self) -> RootNodeEditData {
        let children: Vector<_> = self
            .children
            .iter()
            .map(|node| node.widget_edit_data())
            .collect();

        RootNodeEditData {
            children,
            selected: Vector::new(),
        }
    }
}
