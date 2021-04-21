use crate::session::Session;
use crate::site_modules::Module;
use crate::site_modules::ModuleExt;
use crate::task::Task;
use async_recursion::async_recursion;
use async_std::channel::Sender;
use async_std::path::Path;
use async_std::path::PathBuf;
use async_trait::async_trait;
use config::{Config, ConfigEnum};
use config_derive::Config;
use enum_dispatch::enum_dispatch;
use futures::future::join_all;
use futures::future::try_join_all;
use futures::future::{BoxFuture, FutureExt};
use futures::stream::FuturesUnordered;
use serde::Serialize;
use crate::errors::Error;


#[derive(Config, Clone, Serialize)]
pub struct RootNode {
    #[config(ty = "struct")]
    pub children: Vec<Node>,
}

impl RootNode {
    pub async fn run(&self, session: Session, sender: Sender<Task>) -> Result<(), Error> {
        let futures: Vec<_> = self
            .children
            .iter()
            .map(|child| child.run(session.clone(), sender.clone(), PathBuf::new()))
            .collect();

        try_join_all(futures).await?;
        Ok(())
    }
}


#[derive(Config, Clone, Serialize)]
pub struct Node {
    #[config(ty = "enum")]
    pub ty: NodeType,
    #[config(ty = "struct")]
    pub children: Vec<Node>,

    #[config(ty = "struct")]
    pub meta_data: MetaData,
}

impl Node {
    #[async_recursion]
    async fn run(&self, session: Session, sender: Sender<Task>, base_path: PathBuf) -> Result<(), Error> {
        let segment = self.ty.path_segment(&session).await?;
        if segment.is_absolute() {
            panic!("segment is not allowed to be absolute")
        }
        let path = base_path.join(segment);

        let futures: Vec<_> = self
            .children
            .iter()
            .map(|child| child.run(session.clone(), sender.clone(), path.clone()))
            .collect();

        try_join_all(futures).await?;

        match &self.ty {
            NodeType::Site(site) => site.run(&session, sender, path).await,
            NodeType::Folder(_) => Ok(()),
        }
    }
}

#[async_trait]
#[enum_dispatch]
trait NodeTypeExt {
    async fn path_segment(&self, session: &Session) -> Result<&Path, Error>;
}

#[enum_dispatch(NodeTypeExt)]
#[derive(Config, Clone, Serialize)]
pub enum NodeType {
    #[config(ty = "struct")]
    Folder(Folder),
    #[config(ty = "struct")]
    Site(Site),
}

#[derive(Config, Clone, Serialize)]
pub struct Folder {
    name: String,
}

#[async_trait]
impl NodeTypeExt for Folder {
    async fn path_segment(&self, _session: &Session) -> Result<&Path, Error> {
        Ok(Path::new(&self.name))
    }
}

#[derive(Config, Clone, Serialize)]
pub struct Site {
    #[config(ty = "enum")]
    pub module: Module,
}

impl Site {
    async fn run(&self, session: &Session, sender: Sender<Task>, base_path: PathBuf) -> Result<(), Error> {
        session.login(&self.module).await?;
        self.module.retrieve_urls(session, sender, base_path).await
    }
}

#[async_trait]
impl NodeTypeExt for Site {
    async fn path_segment(&self, session: &Session) -> Result<&Path, Error> {
        self.module.folder_name(session).await
    }
}

#[derive(Config, Clone, Serialize)]
pub struct MetaData {}
