use crate::error::Result;
use crate::session::Session;
use async_recursion::async_recursion;
use config::{Config, ConfigEnum};
use config_derive::Config;
use druid::im::Vector;
use druid::{Data, ExtEventSink, Widget, WidgetExt, WidgetId};
use sha1::Digest;
use std::path::PathBuf;

use futures::future::{join_all, try_join_all};

use crate::settings::DownloadSettings;
use futures::prelude::*;
use serde::Serialize;
use std::sync::Arc;

use crate::template::communication::{Communication, RawCommunication};
use crate::template::node_type::site_data::SiteEvent;
use crate::template::node_type::{NodeType, NodeTypeData};
use crate::template::nodes::node_data::{NodeData, NodeState};
use crate::template::nodes::node_edit_data::NodeEditData;
use crate::utils::spawn_drop;
use crate::widgets::tree::NodeIndex;
use crate::TError;
use std::collections::HashSet;

#[derive(Config, Clone, Serialize, Debug, Data)]
pub struct MetaData {}

#[derive(Debug, PartialEq)]
pub enum Status {
    Success,
    Failure,
}

#[derive(Config, Serialize, Debug)]
pub struct RawNode {
    #[config(ty = "Enum")]
    pub ty: NodeType,
    #[config(ty = "_<Struct>")]
    pub children: Vec<RawNode>,
    #[config(ty = "Struct")]
    pub meta_data: MetaData,

    pub cached_path_segment: Option<PathBuf>,
}

impl RawNode {
    pub fn transform(self, index: NodeIndex, comm: RawCommunication) -> Node {
        Node {
            ty: self.ty,
            children: self
                .children
                .into_iter()
                .enumerate()
                .map(|(idx, raw_node)| {
                    let mut new_index = index.clone();
                    new_index.push(idx);
                    raw_node.transform(new_index, comm.clone())
                })
                .collect(),
            meta_data: self.meta_data,
            cached_path_segment: self.cached_path_segment,
            comm: comm.with_idx(index.clone()),
            path: None,
            index,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Node {
    pub ty: NodeType,
    pub children: Vec<Node>,
    pub meta_data: MetaData,
    pub cached_path_segment: Option<PathBuf>,
    pub comm: Communication,
    pub index: NodeIndex,
    pub path: Option<PathBuf>,
}

impl Node {
    pub fn raw(self) -> RawNode {
        RawNode {
            ty: self.ty,
            children: self.children.into_iter().map(|node| node.raw()).collect(),
            meta_data: self.meta_data,
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
            self.comm.send_event(PathEvent::Cached(path.clone()));
            path
        } else {
            match PathEvent::wrapper(
                async {
                    self.ty
                        .path_segment(&session, &dsettings)
                        .await
                        .map(|segment| {
                            if segment.is_absolute() {
                                panic!("segment is not allowed to be absolute")
                            }
                            base_path.join(segment)
                        })
                },
                &self.comm,
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
            .map(|(idx, child)| child.prepare(session, Arc::clone(&dsettings), path.clone()))
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
                        self.comm.clone(),
                    ),
                );
                futures.push(Box::pin(async move { handle.await.unwrap() }))
            }
        }

        join_all(futures).await;
    }

    pub fn widget_data(&self) -> NodeData {
        let children: Vector<_> = self
            .children
            .iter()
            .map(|node| node.widget_data())
            .collect();
        NodeData {
            expanded: true,
            children,
            meta_data: self.meta_data.clone(),
            cached_path_segment: self.cached_path_segment.clone(),
            ty: self.ty.widget_data(),
            state: NodeState::new(),
            path: None,
        }
    }

    pub fn widget_edit_data(&self) -> NodeEditData {
        let children: Vector<_> = self
            .children
            .iter()
            .map(|node| node.widget_edit_data())
            .collect();
        NodeEditData {
            expanded: true,
            children,
            ty: Some(self.ty.widget_edit_data(self.meta_data.clone())),
        }
    }
}

#[derive(Debug)]
pub enum NodeEvent {
    Path(PathEvent),
    Site(SiteEvent),
}

impl From<PathEvent> for NodeEvent {
    fn from(path_status: PathEvent) -> Self {
        NodeEvent::Path(path_status)
    }
}

impl<T> From<T> for NodeEvent
where
    T: Into<SiteEvent>,
{
    fn from(site_status: T) -> Self {
        NodeEvent::Site(site_status.into())
    }
}

#[derive(Debug)]
pub enum PathEvent {
    Start,
    Cached(PathBuf),
    Finish(PathBuf),
    Err(TError),
}

impl PathEvent {
    pub async fn wrapper(
        inner_fn: impl Future<Output = Result<PathBuf>>,
        comm: &Communication,
    ) -> Option<PathBuf> {
        comm.send_event(Self::Start);
        match inner_fn.await {
            Ok(data) => {
                comm.send_event(Self::Finish(data.clone()));
                Some(data)
            }
            Err(err) => {
                comm.send_event(Self::Err(err));
                None
            }
        }
    }
}
