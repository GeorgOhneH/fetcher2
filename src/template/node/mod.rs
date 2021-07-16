pub mod widget;

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

use crate::template::node::widget::{NodeData, TreeNodeWidget, RootNodeWidget, RootNodeData};
use crate::template::node_type::{NodeType, NodeTypeData};
use crate::template::widget::WidgetCommunication;

#[derive(Config, Serialize, Debug)]
pub struct RootNode {
    #[config(inner_ty = "struct")]
    pub children: Vec<Node>,
}

impl RootNode {
    pub async fn prepare(
        &mut self,
        session: &Session,
        dsettings: Arc<DownloadSettings>,
    ) -> Result<()> {
        let futures: Vec<_> = self
            .children
            .iter_mut()
            .map(|child| child.prepare(session, Arc::clone(&dsettings), PathBuf::new()))
            .collect();

        try_join_all(futures).await?;
        Ok(())
    }

    pub async fn run(&self, session: &Session, dsettings: Arc<DownloadSettings>) -> Result<()> {
        let futures: Vec<_> = self
            .children
            .iter()
            .map(|child| child.run(session, Arc::clone(&dsettings)))
            .collect();

        try_join_all(futures).await?;
        Ok(())
    }

    pub fn widget(&mut self) -> (RootNodeData, RootNodeWidget) {
        let mut widget = RootNodeWidget::new();
        let mut children_data = Vec::new();
        for (child_data, child_widgets) in self
            .children
            .iter_mut()
            .map(|node| node.widget()) {
            children_data.push(child_data);
            widget.add_child(child_widgets);
        }
        let data = RootNodeData {
            children: children_data.into(),
        };
        (data, widget)
    }

    pub fn set_sink(&mut self, sink: ExtEventSink) {
        self.children.iter_mut().map(|node| node.set_sink(sink.clone())).for_each(drop);
    }
}


#[derive(Config, Clone, Serialize, Debug, Data)]
pub struct MetaData {}

#[derive(Config, Serialize, Debug)]
pub struct Node {
    #[config(ty = "enum")]
    pub ty: NodeType,
    #[config(inner_ty = "struct")]
    pub children: Vec<Node>,
    #[config(ty = "struct")]
    pub meta_data: MetaData,

    pub cached_path: Option<PathBuf>,

    #[serde(skip)]
    #[config(skip = WidgetCommunication::new())]
    pub comm: WidgetCommunication,
}

impl Node {
    #[async_recursion]
    async fn prepare<'a>(
        &'a mut self,
        session: &'a Session,
        dsettings: Arc<DownloadSettings>,
        base_path: PathBuf,
    ) -> Result<()> {
        let segment = self.ty.path_segment(&session, &dsettings).await?;
        if segment.is_absolute() {
            panic!("segment is not allowed to be absolute")
        }
        // TODO only when really needed?
        let path = base_path.join(segment);

        let mut futures: Vec<_> = self
            .children
            .iter_mut()
            .map(|child| child.prepare(session, Arc::clone(&dsettings), path.clone()))
            .collect();

        try_join_all(futures).await?;
        self.cached_path = Some(path.clone());
        println!("{:?}", self.comm);
        self.comm.send_new_path(path).await?;
        Ok(())
    }
    #[async_recursion]
    async fn run<'a>(
        &'a self,
        session: &'a Session,
        dsettings: Arc<DownloadSettings>,
    ) -> Result<()> {
        let mut futures: Vec<_> = self
            .children
            .iter()
            .map(|child| child.run(session, Arc::clone(&dsettings)))
            .collect();

        if let NodeType::Site(site) = &self.ty {
            let site_clone = site.clone();
            let handle = tokio::spawn(
                site_clone.run(
                    session.clone(),
                    dsettings,
                    self.cached_path
                        .as_ref()
                        .expect("Called run before prepare")
                        .clone(),
                ),
            );
            futures.push(Box::pin(async move { handle.await? }))
        };

        try_join_all(futures).await?;
        Ok(())
    }

    pub fn widget(&mut self) -> (NodeData, TreeNodeWidget) {
        let widget_id = WidgetId::next();
        let mut widget = TreeNodeWidget::new(true, widget_id.clone());
        self.comm.id = Some(widget_id);

        let mut children_data = Vec::new();
        for (child_data, child_widgets) in self
            .children
            .iter_mut()
            .map(|node| node.widget()) {
            children_data.push(child_data);
            widget.add_child(child_widgets);
        }
        let data = NodeData {
            children: children_data.into(),
            meta_data: self.meta_data.clone(),
            cached_path: self.cached_path.clone(),
            ty: self.ty.widget_data(),
        };
        (data, widget)
    }

    pub fn set_sink(&mut self, sink: ExtEventSink) {
        self.comm.sink = Some(sink.clone());
        self.children.iter_mut().map(|node| node.set_sink(sink.clone())).for_each(drop);
    }
}
