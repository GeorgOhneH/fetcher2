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
use crate::template::node_type::site::{SiteEvent, SiteState};
use crate::template::node_type::{NodeType, NodeTypeData};
use crate::template::nodes::node_data::{NodeData, NodeState};
use crate::template::NodeIndex;
use crate::utils::spawn_drop;
use crate::TError;
use std::collections::HashSet;

#[derive(Config, Clone, Serialize, Debug, Data)]
pub struct MetaData {}

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
            children: self
                .children
                .into_iter()
                .map(|node| node.raw())
                .collect(),
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
    ) -> std::result::Result<(), ()> {
        let segment = if let Some(segment) = &self.cached_path_segment {
            segment.clone()
        } else {
            self.comm.send_event(PathEvent::Start);
            match self.ty.path_segment(&session, &dsettings).await {
                Ok(segment) => segment,
                Err(err) => {
                    self.comm.send_event(PathEvent::Err(err));
                    return Err(());
                }
            }
        };

        if segment.is_absolute() {
            panic!("segment is not allowed to be absolute")
        }

        let path = base_path.join(segment);
        self.path = Some(path.clone());
        if let Some(_) = &self.cached_path_segment {
            self.comm.send_event(PathEvent::Cached(path.clone()));
        } else {
            self.comm.send_event(PathEvent::Finish(path.clone()));
        }

        let futures: Vec<_> = self
            .children
            .iter_mut()
            .enumerate()
            .map(|(idx, child)| child.prepare(session, Arc::clone(&dsettings), path.clone()))
            .collect();

        if join_all(futures).await.iter().any(|r| r.is_err()) {
            Err(())
        } else {
            Ok(())
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
            path: None
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
