pub mod communication;
pub mod node_type;
pub mod nodes;
pub mod widget;

use crate::error::{Result, TError};
use crate::session::Session;
use crate::site_modules::Mode as PolyboxMode;
use crate::site_modules::Module;
use crate::site_modules::{Minimal, Polybox};
use crate::task::Task;
pub use crate::template::node_type::{DownloadArgs, Extensions, Mode};
use druid::{Data, ExtEventSink, Lens, WidgetExt, WidgetId};
use tokio::io::AsyncWriteExt;

use crate::settings::DownloadSettings;
use crate::template::communication::{Communication, RawCommunication};
use crate::template::node_type::{NodeType, Site, SiteStorage};
use crate::template::nodes::node::{MetaData, Node, RawNode};
use crate::template::nodes::root::{RootNode, RawRootNode};
use crate::template::widget::{TemplateData};
use config::{Config, ConfigEnum};
use druid::widget::prelude::*;
use druid::widget::Label;
use std::collections::{HashMap, HashSet};
use std::fmt::{Debug, Formatter};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex, RwLock};
use tokio::fs;
use crate::ui::TemplateInfoSelect;
use crate::widgets::tree::NodeIndex;

#[derive(Debug)]
pub struct Template {
    root: RootNode,
    raw_comm: RawCommunication,
}

impl Template {
    pub fn new(comm: RawCommunication) -> Self {
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
                        storage: SiteStorage {
                            files: dashmap::DashMap::new(),
                            history: Mutex::new(Vec::new()),
                        },
                        download_args: None,
                    })),
                    children: vec![RawNode {
                        cached_path_segment: None,
                        ty: NodeType::Site(Arc::new(Site {
                            module: Module::Minimal(Minimal { parameters: None }),
                            storage: SiteStorage {
                                files: dashmap::DashMap::new(),
                                history: Mutex::new(Vec::new()),
                            },
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
                        storage: SiteStorage {
                            files: dashmap::DashMap::new(),
                            history: Mutex::new(Vec::new()),
                        },
                        download_args: None,
                    })),
                    children: vec![].into(),
                    meta_data: MetaData {},
                },
                // Node {
                //     ty: NodeType::Site(Arc::new(Site {
                //         module: Module::Polybox(Polybox {
                //             id: "sYrHnA3ZBbuDcip".to_owned(),
                //             mode: PolyboxMode::Shared(None),
                //         }),
                //         storage: SiteStorage {
                //             files: dashmap::DashMap::new(),
                //         },
                //         download_args: None,
                //     })),
                //     children: vec![],
                //     meta_data: MetaData {},
                // },
            ]
            .into(),
        };

        file_root.update_app(&mut raw_app).unwrap();
        let raw_root = RawRootNode::parse_from_app(&raw_app).unwrap();

        Self {
            root: raw_root.transform(comm.clone()),
            raw_comm: comm,
        }
    }

    pub async fn prepare(&mut self, dsettings: Arc<DownloadSettings>) -> std::result::Result<(), ()> {
        let session = Session::new();
        self.root.prepare(&session, dsettings).await
    }

    pub async fn run_root(&self, dsettings: Arc<DownloadSettings>) {
        let session = Session::new();
        self.root.run(&session, dsettings, None).await
    }

    pub async fn run(&self, dsettings: Arc<DownloadSettings>, indexes: Option<HashSet<NodeIndex>>) {
        let session = Session::new();
        self.root.run(&session, dsettings, indexes.as_ref()).await
    }

    pub async fn save(&self, path: &Path) -> Result<()> {
        let template_str = serde_yaml::to_string(&self.root.clone().raw())?;

        let mut f = fs::OpenOptions::new()
            .write(true)
            .truncate(true)
            .create(true)
            .open(path)
            .await?;
        f.write_all(&template_str.as_bytes()).await?;

        f.shutdown().await?;
        Ok(())
    }

    pub async fn load(&mut self, path: &Path) -> Result<()> {
        let x = String::from_utf8(fs::read(path).await?)?;
        let raw_root = RawRootNode::load_from_str(&*x)?;
        self.root = raw_root.transform(self.raw_comm.clone());
        Ok(())
    }

    pub fn widget_data(&self) -> TemplateData {
        TemplateData {
            root: self.root.widget_data(),
        }
    }
}
