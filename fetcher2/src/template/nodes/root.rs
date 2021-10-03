use std::collections::HashSet;
use std::path::PathBuf;
use std::sync::Arc;

use futures::future::join_all;
use serde::Deserialize;
use serde::Serialize;

use crate::session::Session;
use crate::settings::DownloadSettings;
use crate::template::communication::{CommunicationExt, RawCommunicationExt};
use crate::template::nodes::node::{Node, RawNode, Status};
use crate::template::NodeIndex;

#[derive(Serialize, Deserialize, Debug)]
pub struct RawRootNode {
    pub children: Vec<RawNode>,
}

impl RawRootNode {
    pub fn transform<T: CommunicationExt>(self, comm: impl RawCommunicationExt<T>) -> RootNode<T> {
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
pub struct RootNode<T> {
    pub children: Vec<Node<T>>,
}

impl<T: CommunicationExt> RootNode<T> {
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
}
