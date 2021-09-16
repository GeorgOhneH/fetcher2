use std::collections::{HashMap, HashSet};
use std::fmt::{Debug, Formatter};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex, RwLock};

use config::{Config, ConfigEnum};
use druid::widget::prelude::*;
use druid::widget::Label;
use druid::{Data, ExtEventSink, Lens, WidgetExt, WidgetId};
use tokio::fs;
use tokio::io::AsyncWriteExt;

use crate::error::{Result, TError};
use crate::session::Session;
use crate::settings::DownloadSettings;
use crate::site_modules::Mode as PolyboxMode;
use crate::site_modules::Module;
use crate::site_modules::{Minimal, Polybox};
use crate::task::Task;
use crate::template::communication::{Communication, RawCommunication};
pub use crate::template::node_type::{DownloadArgs, Extensions, Mode};
use crate::template::node_type::{NodeType, Site, SiteStorage};
use crate::template::nodes::node::{MetaData, Node, RawNode, Status};
use crate::template::nodes::root::{RawRootNode, RootNode};
use crate::template::nodes::root_edit_data::RootNodeEditData;
use crate::template::widget_data::TemplateData;
use crate::template::widget_edit_data::TemplateEditData;
use crate::ui::TemplateInfoSelect;
use crate::widgets::tree::NodeIndex;

pub mod communication;
pub mod node_type;
pub mod nodes;
pub mod widget_data;
pub mod widget_edit_data;

#[derive(Debug)]
pub struct Template {
    root: RootNode,
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

    pub fn test(comm: RawCommunication) -> Self {
        let mut raw_app = RawNode::builder().build();

        let file_root = RawRootNode {
            children: vec![
                RawNode {
                    cached_path_segment: None,
                    ty: NodeType::Site(Arc::new(Site {
                        module: Module::Polybox(Polybox {
                            id: "TnFKtU4xoe5gIZy".to_owned(),
                            mode: PolyboxMode::Shared(Some("123".to_owned())),
                        }),
                        storage: Arc::new(SiteStorage {
                            files: dashmap::DashMap::new(),
                            history: Mutex::new(Vec::new()),
                        }),
                        download_args: None,
                    })),
                    children: vec![RawNode {
                        cached_path_segment: None,
                        ty: NodeType::Site(Arc::new(Site {
                            module: Module::Minimal(Minimal { parameters: None }),
                            storage: Arc::new(SiteStorage {
                                files: dashmap::DashMap::new(),
                                history: Mutex::new(Vec::new()),
                            }),
                            download_args: None,
                        })),
                        children: vec![].into(),
                        meta_data: MetaData {},
                    }]
                    .into(),
                    meta_data: MetaData {},
                },
                RawNode {
                    cached_path_segment: None,
                    ty: NodeType::Site(Arc::new(Site {
                        module: Module::Polybox(Polybox {
                            id: "1929777502".to_owned(),
                            mode: PolyboxMode::Private,
                        }),
                        storage: Arc::new(SiteStorage {
                            files: dashmap::DashMap::new(),
                            history: Mutex::new(Vec::new()),
                        }),
                        download_args: None,
                    })),
                    children: vec![].into(),
                    meta_data: MetaData {},
                },
            ]
            .into(),
        };

        file_root.update_app(&mut raw_app).unwrap();
        let raw_root = RawRootNode::parse_from_app(&raw_app).unwrap();

        Self {
            root: raw_root.transform(comm),
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
        let x = String::from_utf8(fs::read(path).await?)?;
        let raw_root: RawRootNode = ron::from_str(&*x)?;
        Ok(Self {
            root: raw_root.transform(comm),
            is_prepared: false,
            save_path: Some(path.to_owned()),
        })
    }

    pub fn is_prepared(&self) -> bool {
        self.is_prepared
    }

    pub async fn prepare(&mut self, dsettings: Arc<DownloadSettings>) -> Status {
        // since only one mutable reference is allowed this cant
        // run in parallel and self.is_prepared is always correct
        if self.is_prepared {
            return Status::Success;
        }
        let session = Session::new();
        let r = self.root.prepare(&session, dsettings).await;
        if let Status::Success = r {
            self.is_prepared = true;
        }
        r
    }

    pub async fn run_root(&self, dsettings: Arc<DownloadSettings>) {
        let session = Session::new();
        self.root.run(&session, dsettings, None).await
    }

    pub async fn run(&self, dsettings: Arc<DownloadSettings>, indexes: &HashSet<NodeIndex>) {
        let session = Session::new();
        self.root.run(&session, dsettings, Some(indexes)).await
    }

    pub fn inform_of_cancel(&self) {
        self.root.inform_of_cancel()
    }

    pub async fn save(&self) -> Result<()> {
        if let Some(save_path) = &self.save_path {
            let template_str = ron::to_string(&self.root.clone().raw())?;

            let mut f = fs::OpenOptions::new()
                .write(true)
                .truncate(true)
                .create(true)
                .open(save_path)
                .await?;
            f.write_all(&template_str.as_bytes()).await?;

            f.shutdown().await?;
        }
        Ok(())
    }

    pub fn widget_data(&self) -> TemplateData {
        TemplateData {
            root: self.root.widget_data(),
            save_path: self.save_path.clone(),
        }
    }

    pub fn widget_edit_data(&self) -> TemplateEditData {
        TemplateEditData {
            root: self.root.widget_edit_data(),
            save_path: self.save_path.clone(),
        }
    }
}
