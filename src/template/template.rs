use crate::errors::TemplateError;
use crate::session::Session;
use crate::site_modules::Minimal;
use crate::site_modules::Module;
use crate::task::Task;
use tokio::io::AsyncWriteExt;

use crate::settings::DownloadSettings;
use crate::template::node::{MetaData, Node, NodeType, Site};
use crate::template::node::{RootNode, SiteStorage};
use async_std::channel::Sender;
use config::{Config, ConfigEnum};
use std::collections::HashMap;
use std::sync::{Mutex, RwLock, Arc};
use std::path::Path;
use tokio::fs;

pub struct Template {
    root: RootNode,
}

impl Template {
    pub fn new() -> Self {
        let mut app = RootNode::build_app();

        let root = RootNode {
            children: vec![Node {
                ty: NodeType::Site(Arc::new(Site {
                    module: Module::Minimal(Minimal { parameters: None }),
                    storage: SiteStorage {
                        files: dashmap::DashMap::new()
                    },
                    download_args: None,
                })),
                children: vec![Node {
                    ty: NodeType::Site(Arc::new(Site {
                        module: Module::Minimal(Minimal { parameters: None }),
                        storage: SiteStorage {
                            files: dashmap::DashMap::new()
                        },
                        download_args: None,
                    })),
                    children: vec![],
                    meta_data: MetaData {},
                }],
                meta_data: MetaData {},
            }],
        };

        root.update_app(&mut app).unwrap();

        Self {
            root: RootNode::parse_from_app(&app).unwrap(),
        }
    }
    pub async fn run_root(
        &self,
        session: Session,
        dsettings: DownloadSettings,
    ) -> Result<(), TemplateError> {
        self.root.run(&session, Arc::new(dsettings)).await
    }

    pub async fn save(&self, path: &Path) -> Result<(), TemplateError> {
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

    pub async fn load(&mut self, path: &Path) -> Result<(), TemplateError> {
        let x = String::from_utf8(fs::read(path).await?)?;
        self.root = RootNode::load_from_str(&*x)?;
        Ok(())

    }
}
