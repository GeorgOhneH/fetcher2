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
use crate::template::communication::WidgetCommunication;
use crate::template::node_type::{NodeType, Site, SiteStorage};
use crate::template::nodes::node::{MetaData, Node, PrepareStatus};
use crate::template::nodes::root::RootNode;
use crate::template::widget::{TemplateData, TemplateWidget};
use config::{Config, ConfigEnum};
use druid::widget::prelude::*;
use druid::widget::Label;
use std::collections::{HashMap, HashSet};
use std::fmt::{Debug, Formatter};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex, RwLock};
use tokio::fs;

#[derive(Debug)]
pub struct Template {
    root: RootNode,
    comm: WidgetCommunication,
}

pub type NodeIndex = Vec<usize>;

impl Template {
    pub fn new() -> Self {
        let mut app = Node::builder().build();

        let root = RootNode {
            children: vec![
                Node {
                    cached_path: None,
                    comm: WidgetCommunication::new(),
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
                    children: vec![Node {
                        cached_path: None,
                        comm: WidgetCommunication::new(),
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
                        index: Vec::new(),
                    }]
                    .into(),
                    meta_data: MetaData {},
                    index: Vec::new(),
                },
                Node {
                    cached_path: None,
                    comm: WidgetCommunication::new(),
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
                    index: Vec::new(),
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

        root.update_app(&mut app).unwrap();

        Self {
            root: RootNode::parse_from_app(&app).unwrap(),
            comm: WidgetCommunication::new(),
        }
    }

    pub async fn prepare(&mut self, dsettings: Arc<DownloadSettings>) -> Result<PrepareStatus> {
        let session = Session::new();
        self.root.prepare(&session, dsettings).await
    }

    pub async fn run_root(&self, dsettings: Arc<DownloadSettings>) -> Result<()> {
        let session = Session::new();
        self.root.run(&session, dsettings, None).await
    }

    pub async fn run(&self, dsettings: Arc<DownloadSettings>, indexes: Option<HashSet<NodeIndex>>) -> Result<()> {
        let session = Session::new();
        self.root.run(&session, dsettings, indexes.as_ref()).await
    }

    pub async fn save(&self, path: &Path) -> Result<()> {
        let template_str = serde_yaml::to_string(&self.root)?;

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
        self.root = RootNode::load_from_str(&*x)?;
        Ok(())
    }

    pub fn widget(&mut self) -> (TemplateData, TemplateWidget) {
        let (root_data, root_widget) = self.root.widget();
        let data = TemplateData { root: root_data, selected: None };
        let header = vec![
            Label::new("Name").boxed(),
            Label::new("Added|Replaced 0|0").align_right().boxed(),
            Label::new("Status").boxed(),
        ];
        (data, TemplateWidget::new(root_widget, header))
    }

    pub fn set_sink(&mut self, sink: ExtEventSink) {
        self.comm.sink = Some(sink.clone());
        self.root.set_sink(sink);
    }
}
