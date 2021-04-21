use crate::errors::Error;
use crate::session::Session;
use crate::site_modules::Minimal;
use crate::site_modules::{Module, ModuleExt};
use crate::task::Task;
use crate::template::node::NodeType::Folder;
use crate::template::node::RootNode;
use crate::template::node::{MetaData, Node, NodeType, Site};
use async_std::channel::Sender;
use config::{Config, ConfigEnum};
use config_derive::Config;
use serde::Serialize;

pub struct Template {
    root: RootNode,
}

impl Template {
    pub fn new() -> Self {
        let mut app = RootNode::build_app();

        let root = RootNode {
            children: vec![Node {
                ty: NodeType::Site(Site {
                    module: Module::Minimal(Minimal { parameters: None }),
                }),
                children: vec![Node {
                    ty: NodeType::Site(Site {
                        module: Module::Minimal(Minimal { parameters: None }),
                    }),
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
    pub async fn run_root(&self, session: Session, sender: Sender<Task>) -> Result<(), Error> {
        self.root.run(session, sender).await
    }
}
