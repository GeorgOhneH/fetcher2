use crate::errors::TemplateError;
use crate::session::Session;
use crate::site_modules::Minimal;
use crate::site_modules::Module;
use crate::task::Task;

use crate::settings::DownloadSettings;
use crate::template::node::{MetaData, Node, NodeType, Site};
use crate::template::node::{RootNode, SiteStorage};
use async_std::channel::Sender;
use config::{Config, ConfigEnum};

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
                    storage: SiteStorage {},
                    download_args: None,
                }),
                children: vec![Node {
                    ty: NodeType::Site(Site {
                        module: Module::Minimal(Minimal { parameters: None }),
                        storage: SiteStorage {},
                        download_args: None,
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
    pub async fn run_root(
        &self,
        session: Session,
        dsettings: &DownloadSettings,
    ) -> Result<(), TemplateError> {
        self.root.run(&session, dsettings).await
    }
}
