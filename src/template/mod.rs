use std::collections::{HashMap, HashSet};
use std::fmt::{Debug, Formatter};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex, RwLock};

use config::{Config, ConfigEnum};
use druid::{Data, ExtEventSink, Lens, WidgetExt, WidgetId};
use druid::widget::Label;
use druid::widget::prelude::*;
use tokio::fs;
use tokio::io::AsyncWriteExt;

use crate::data::settings::DownloadSettings;
use crate::error::{Result, TError};
use crate::session::Session;
use crate::site_modules::{Minimal, Polybox};
use crate::site_modules::Mode as PolyboxMode;
use crate::site_modules::Module;
use crate::task::Task;
use crate::template::communication::{Communication, RawCommunication};
pub use crate::template::node_type::{DownloadArgs, Extensions, Mode};
use crate::template::node_type::{NodeType, Site, SiteStorage};
use crate::template::node_type::site::{FileData, MsgKind, TaskMsg};
use crate::template::nodes::node::{Node, RawNode, Status};
use crate::template::nodes::root::{RawRootNode, RootNode};
use crate::template::nodes::root_data::RootNodeData;
use crate::template::nodes::root_edit_data::RootNodeEditData;
use crate::template::widget_data::TemplateData;
use crate::template::widget_edit_data::TemplateEditData;
use crate::widgets::tree::NodeIndex;

pub mod communication;
pub mod node_type;
pub mod nodes;
pub mod widget_data;
pub mod widget_edit_data;

#[derive(Debug)]
pub struct Template {
    pub root: RootNode,
    save_path: Option<PathBuf>,
    is_prepared: bool,
}

impl Template {
    pub fn new() -> Self {
        Self {
            root: RootNode::new(),
            is_prepared: false,
            save_path: None,
        }
    }

    pub fn from_raw(
        edit_data: RootNodeEditData,
        comm: RawCommunication,
        save_path: PathBuf,
    ) -> Self {
        let raw_root = edit_data.raw();
        Self {
            root: raw_root.transform(comm),
            is_prepared: false,
            save_path: Some(save_path),
        }
    }

    pub async fn load(path: &Path, comm: RawCommunication) -> Result<Self> {
        dbg!("load");
        let x = String::from_utf8(fs::read(path).await?)?;
        dbg!("build");
        let raw_root: RawRootNode = ron::from_str(&*x)?;
        dbg!("build finished");
        Ok(Self {
            root: raw_root.transform(comm),
            is_prepared: false,
            save_path: Some(path.to_owned()),
        })
    }

    pub fn is_prepared(&self) -> bool {
        self.is_prepared
    }

    pub fn inform_of_cancel(&self) {
        self.root.inform_of_cancel()
    }

    pub async fn prepare(&mut self, dsettings: Arc<DownloadSettings>) -> Status {
        // since only one mutable reference is allowed this cant
        // run in parallel and self.is_prepared is always correct
        if self.is_prepared {
            return Status::Success;
        }
        let session = Session::new();
        let status = self.root.prepare(&session, dsettings).await;
        if let Status::Success = status {
            self.is_prepared = true;
        }
        status
    }

    pub async fn run_root(&self, dsettings: Arc<DownloadSettings>) {
        let session = Session::new();
        self.root.run(&session, dsettings, None).await
    }

    pub async fn run(&self, dsettings: Arc<DownloadSettings>, indexes: &HashSet<NodeIndex>) {
        let session = Session::new();
        self.root.run(&session, dsettings, Some(indexes)).await
    }

    pub async fn save(&self) -> Result<()> {
        if let Some(save_path) = &self.save_path {
            // TODO remove later
            let raw_root = self.root.clone().raw();
            let template_str = ron::to_string(&raw_root)?;
            let mut f = fs::OpenOptions::new()
                .write(true)
                .truncate(true)
                .create(true)
                .open(save_path)
                .await?;
            f.write_all(&template_str.as_bytes()).await?;

            f.shutdown().await?;

            let test_raw_root: RawRootNode = ron::from_str(&template_str).unwrap();
            assert_eq!(raw_root, test_raw_root);
        }
        Ok(())
    }

    pub fn widget_data(&self) -> (RootNodeData, Option<PathBuf>) {
        (self.root.widget_data(), self.save_path.clone())
    }

    pub fn widget_edit_data(&self) -> (RootNodeEditData, Option<PathBuf>) {
        (self.root.widget_edit_data(), self.save_path.clone())
    }
}
