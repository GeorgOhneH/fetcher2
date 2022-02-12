use std::collections::HashSet;
use std::path::PathBuf;
use std::sync::Arc;

use async_recursion::async_recursion;
use futures::future::join_all;
use futures::prelude::*;
use serde::Deserialize;
use serde::Serialize;
use tokio::sync::mpsc::Sender;

use crate::error::Result;
use crate::session::Session;
use crate::settings::DownloadSettings;
use crate::template::communication::{CommunicationExt, RawCommunicationExt, RootNotifier};
use crate::template::node_type::site::SiteEventKind;
use crate::template::node_type::NodeType;
use crate::template::NodeIndex;
use crate::utils::spawn_drop;
use crate::TError;

#[derive(Debug, PartialEq)]
pub enum Status {
    Success,
    Failure,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct RawNode {
    pub ty: NodeType,
    pub children: Vec<RawNode>,

    pub cached_path_segment: Option<PathBuf>,
}

impl RawNode {
    pub fn transform(self, index: NodeIndex, tx: Sender<NodeEvent>) -> Node {
        Node {
            ty: self.ty,
            children: self
                .children
                .into_iter()
                .enumerate()
                .map(|(idx, raw_node)| {
                    let mut new_index = index.clone();
                    new_index.push_back(idx);
                    raw_node.transform(new_index, tx.clone())
                })
                .collect(),
            cached_path_segment: self.cached_path_segment,
            tx: RootNotifier::new(tx, index.clone()),
            path: None,
            index,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Node {
    pub ty: NodeType,
    pub children: Vec<Node>,
    pub cached_path_segment: Option<PathBuf>,
    pub tx: RootNotifier,
    pub index: NodeIndex,
    pub path: Option<PathBuf>,
}

impl Node {
    pub fn raw(self) -> RawNode {
        RawNode {
            ty: self.ty,
            children: self.children.into_iter().map(|node| node.raw()).collect(),
            cached_path_segment: self.cached_path_segment,
        }
    }

    #[async_recursion]
    pub async fn prepare<'a>(
        &'a mut self,
        session: &'a Session,
        dsettings: Arc<DownloadSettings>,
        base_path: PathBuf,
    ) -> Status {
        let path = if let Some(segment) = &self.cached_path_segment {
            if segment.is_absolute() {
                panic!("segment is not allowed to be absolute")
            }
            let path = base_path.join(segment);
            self.tx.notify(PathEventKind::Cached(path.clone())).await;
            path
        } else {
            match PathEventKind::wrapper(
                async {
                    self.ty
                        .path_segment(session, &dsettings)
                        .await
                        .map(|segment| {
                            if segment.is_absolute() {
                                panic!("segment is not allowed to be absolute")
                            }
                            base_path.join(segment)
                        })
                },
                &self.tx,
            )
            .await
            {
                Some(path) => path,
                None => return Status::Failure,
            }
        };

        self.path = Some(path.clone());

        let futures: Vec<_> = self
            .children
            .iter_mut()
            .enumerate()
            .map(|(_idx, child)| child.prepare(session, Arc::clone(&dsettings), path.clone()))
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
    #[async_recursion]
    pub async fn run<'a>(
        &'a self,
        session: &'a Session,
        dsettings: Arc<DownloadSettings>,
        indexes: Option<&'a HashSet<NodeIndex>>,
    ) {
        let mut futures: Vec<_> = self
            .children
            .iter()
            .map(|child| child.run(session, Arc::clone(&dsettings), indexes))
            .collect();

        if indexes.map_or(true, |indexes| indexes.contains(&self.index)) {
            if let NodeType::Site(site) = &self.ty {
                let site_clone = site.clone();
                let handle = spawn_drop(
                    site_clone.run(
                        session.clone(),
                        dsettings,
                        self.path
                            .as_ref()
                            .expect("Called run before prepare")
                            .clone(),
                        self.tx.clone(),
                    ),
                );
                futures.push(Box::pin(async move { handle.await.unwrap() }))
            }
        }

        join_all(futures).await;
    }

    #[async_recursion]
    pub async fn inform_of_cancel(&self) {
        self.tx.notify(NodeEventKind::Canceled).await;
        let futures = self.children.iter().map(|child| child.inform_of_cancel());
        join_all(futures).await;
    }
}


#[derive(Debug)]
pub struct NodeEvent {
    pub kind: NodeEventKind,
    pub idx: NodeIndex,
}

impl NodeEvent {
    pub fn new(kind: NodeEventKind, idx: NodeIndex) -> Self {
        Self {
            kind,
            idx,
        }
    }
}

#[derive(Debug)]
pub enum NodeEventKind {
    Path(PathEventKind),
    Site(SiteEventKind),
    Canceled,
}

impl From<PathEventKind> for NodeEventKind {
    fn from(path_status: PathEventKind) -> Self {
        NodeEventKind::Path(path_status)
    }
}

impl<T> From<T> for NodeEventKind
where
    T: Into<SiteEventKind>,
{
    fn from(site_status: T) -> Self {
        NodeEventKind::Site(site_status.into())
    }
}

#[derive(Debug)]
pub enum PathEventKind {
    Start,
    Cached(PathBuf),
    Finish(PathBuf),
    Err(TError),
}

impl PathEventKind {
    pub fn is_start(&self) -> bool {
        matches!(self, Self::Start | Self::Cached(_))
    }
    pub async fn wrapper(
        inner_fn: impl Future<Output = Result<PathBuf>>,
        tx: &RootNotifier,
    ) -> Option<PathBuf> {
        tx.notify(Self::Start).await;
        match inner_fn.await {
            Ok(data) => {
                tx.notify(Self::Finish(data.clone())).await;
                Some(data)
            }
            Err(err) => {
                tx.notify(Self::Err(err)).await;
                None
            }
        }
    }
}
