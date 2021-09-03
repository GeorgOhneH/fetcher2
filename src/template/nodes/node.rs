use crate::error::Result;
use crate::session::Session;
use async_recursion::async_recursion;
use config::{Config, ConfigEnum};
use config_derive::Config;
use druid::im::Vector;
use druid::{Data, ExtEventSink, Widget, WidgetExt, WidgetId};
use sha1::Digest;
use std::path::PathBuf;

use futures::future::try_join_all;

use crate::settings::DownloadSettings;
use futures::prelude::*;
use serde::Serialize;
use std::sync::Arc;

use crate::template::communication::WidgetCommunication;
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
pub struct Node {
    #[config(ty = "Enum")]
    pub ty: NodeType,
    #[config(ty = "_<Struct>")]
    pub children: Vec<Node>,
    #[config(ty = "Struct")]
    pub meta_data: MetaData,

    pub cached_path: Option<PathBuf>,

    #[serde(skip)]
    #[config(skip = WidgetCommunication::new())]
    pub comm: WidgetCommunication,

    // Will be set after prepare
    #[serde(skip)]
    #[config(skip = Vec::new())]
    pub index: NodeIndex,
}

#[derive(PartialEq)]
pub enum PrepareStatus {
    Success,
    Failure,
}

impl Node {
    #[async_recursion]
    pub async fn prepare<'a>(
        &'a mut self,
        session: &'a Session,
        dsettings: Arc<DownloadSettings>,
        base_path: PathBuf,
        index: NodeIndex,
    ) -> Result<PrepareStatus> {
        self.index = index.clone();

        if let Some(path) = &self.cached_path {
            self.comm.send_event(PathEvent::Cached(path.clone()))?;
            return Ok(PrepareStatus::Success);
        }

        self.comm.send_event(PathEvent::Start)?;

        let segment = match self.ty.path_segment(&session, &dsettings).await {
            Ok(segment) => segment,
            Err(err) => {
                self.comm.send_event(PathEvent::Err(err))?;
                return Ok(PrepareStatus::Failure);
            }
        };
        if segment.is_absolute() {
            panic!("segment is not allowed to be absolute")
        }

        let path = base_path.join(segment);
        self.cached_path = Some(path.clone());
        self.comm.send_event(PathEvent::Finish(path.clone()))?;

        let index_clone = self.index.clone();
        let futures: Vec<_> = self
            .children
            .iter_mut()
            .enumerate()
            .map(|(idx, child)| {
                let mut child_index = index_clone.clone();
                child_index.push(idx);
                child.prepare(session, Arc::clone(&dsettings), path.clone(), child_index)
            })
            .collect();

        if try_join_all(futures)
            .await?
            .iter()
            .any(|status| status == &PrepareStatus::Failure)
        {
            Ok(PrepareStatus::Failure)
        } else {
            Ok(PrepareStatus::Success)
        }
    }
    #[async_recursion]
    pub async fn run<'a>(
        &'a self,
        session: &'a Session,
        dsettings: Arc<DownloadSettings>,
        indexes: Option<&'a HashSet<NodeIndex>>,
    ) -> Result<()> {
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
                        self.cached_path
                            .as_ref()
                            .expect("Called run before prepare")
                            .clone(),
                        self.comm.clone(),
                    ),
                );
                futures.push(Box::pin(async move { handle.await? }))
            }
        }

        try_join_all(futures).await?;
        Ok(())
    }

    pub fn set_sink(&mut self, sink: ExtEventSink) {
        self.comm.sink = Some(sink.clone());
        self.children
            .iter_mut()
            .map(|node| node.set_sink(sink.clone()))
            .for_each(drop);
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
