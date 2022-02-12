use std::collections::HashSet;
use std::path::PathBuf;
use std::sync::Arc;

use futures::future::join_all;
use im::Vector;
use serde::Deserialize;
use serde::Serialize;
use tokio::sync::mpsc::Sender;

use crate::session::Session;
use crate::settings::DownloadSettings;
use crate::template::communication::{CommunicationExt, RawCommunicationExt};
use crate::template::nodes::node::{Node, NodeEvent, NodeEventKind, RawNode, Status};
use crate::template::NodeIndex;

#[derive(Serialize, Deserialize, Debug)]
pub struct RawRootNode {
    pub children: Vec<RawNode>,
}

impl RawRootNode {
    pub fn transform(self, tx: Sender<NodeEvent>) -> RootNode {
        RootNode {
            children: self
                .children
                .into_iter()
                .enumerate()
                .map(|(idx, raw_node)| {
                    let mut node_idx = Vector::new();
                    node_idx.push_back(idx);
                    raw_node.transform(node_idx, tx.clone())
                })
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
            .map(|(_idx, child)| child.prepare(session, Arc::clone(&dsettings), PathBuf::new()))
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
        let futures = self
            .children
            .iter()
            .map(|child| child.run(session, Arc::clone(&dsettings), indexes));

        join_all(futures).await;
    }

    pub async fn inform_of_cancel(&self) {
        let futures = self.children.iter().map(|child| child.inform_of_cancel());
        join_all(futures).await;
    }
}
