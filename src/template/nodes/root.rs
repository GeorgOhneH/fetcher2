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

use crate::template::node_type::{NodeType, NodeTypeData};
use crate::template::nodes::node::{Node, PrepareStatus};
use crate::template::nodes::node_widget::NodeWidget;
use crate::template::nodes::root_widget::{RootNodeData, RootNodeWidget};
use crate::template::NodeIndex;
use std::collections::HashSet;

#[derive(Config, Serialize, Debug)]
pub struct RootNode {
    #[config(ty = "Vec<Struct>")]
    pub children: Vec<Node>,
}

impl RootNode {
    pub async fn prepare(
        &mut self,
        session: &Session,
        dsettings: Arc<DownloadSettings>,
    ) -> Result<PrepareStatus> {
        let futures: Vec<_> = self
            .children
            .iter_mut()
            .enumerate()
            .map(|(idx, child)| {
                child.prepare(session, Arc::clone(&dsettings), PathBuf::new(), vec![idx])
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

    pub async fn run(
        &self,
        session: &Session,
        dsettings: Arc<DownloadSettings>,
        indexes: Option<&HashSet<NodeIndex>>,
    ) -> Result<()> {
        let futures: Vec<_> = self
            .children
            .iter()
            .map(|child| child.run(session, Arc::clone(&dsettings), indexes))
            .collect();

        try_join_all(futures).await?;
        Ok(())
    }

    pub fn widget(&mut self) -> (RootNodeData, RootNodeWidget) {
        let mut widget = RootNodeWidget::new();

        let (data, children): (Vec<_>, Vec<_>) =
            self.children.iter_mut().map(|node| node.widget()).unzip();

        widget.add_children(children);

        let datum = RootNodeData {
            children: data.into(),
        };
        (datum, widget)
    }

    pub fn set_sink(&mut self, sink: ExtEventSink) {
        self.children
            .iter_mut()
            .map(|node| node.set_sink(sink.clone()))
            .for_each(drop);
    }
}
