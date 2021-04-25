use crate::errors::TemplateError;
use crate::session::Session;
use crate::site_modules::Module;
use crate::task::Task;
use async_recursion::async_recursion;
use async_std::path::Path;
use async_std::path::PathBuf;
use async_trait::async_trait;
use config::{Config, ConfigEnum};
use config_derive::Config;
use enum_dispatch::enum_dispatch;

use futures::future::try_join_all;

use crate::settings::DownloadSettings;
use async_std::channel::{self, Receiver, Sender};
use futures::join;
use futures::prelude::*;
use serde::Serialize;
use std::collections::HashMap;
use std::sync::Arc;

use futures::pin_mut;
use futures::stream::{FuturesUnordered, TryStreamExt};

#[derive(Config, Clone, Serialize)]
pub struct RootNode {
    #[config(ty = "struct")]
    pub children: Vec<Node>,
}

impl RootNode {
    pub async fn run(&self, session: &Session, dsettings: &DownloadSettings) -> Result<(), TemplateError> {
        let futures: Vec<_> = self
            .children
            .iter()
            .map(|child| child.run(session, dsettings, PathBuf::new()))
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
    async fn run<'a>(
        &'a self,
        session: &'a Session,
        dsettings: &'a DownloadSettings,
        base_path: PathBuf,
    ) -> Result<(), TemplateError> {
        let segment = self.ty.path_segment(&session).await?;
        if segment.is_absolute() {
            panic!("segment is not allowed to be absolute")
        }
        let path = base_path.join(segment);

        let mut futures: Vec<_> = self
            .children
            .iter()
            .map(|child| child.run(session, dsettings, path.clone()))
            .collect();

        if let NodeType::Site(site) = &self.ty {
            futures.push(Box::pin(site.run(session, dsettings, path)))
        };

        try_join_all(futures).await?;
        Ok(())
    }
}

#[async_trait]
#[enum_dispatch]
trait NodeTypeExt {
    async fn path_segment(&self, session: &Session) -> Result<&Path, TemplateError>;
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
    async fn path_segment(&self, _session: &Session) -> Result<&Path, TemplateError> {
        Ok(Path::new(&self.name))
    }
}

#[derive(Config, Clone, Serialize)]
pub struct Site {
    #[config(ty = "enum")]
    pub module: Module,

    #[config(ty = "struct")]
    pub storage: SiteStorage,

    #[config(ty = "struct")]
    pub download_args: Option<DownloadArgs>,
}

impl Site {
    async fn run(
        &self,
        session: &Session,
        dsettings: &DownloadSettings,
        base_path: PathBuf,
    ) -> Result<(), TemplateError> {
        session.login(&self.module, dsettings).await?;

        let (sender, receiver) = async_std::channel::bounded(100);

        let task_stream = self
            .module
            .retrieve_urls(session.clone(), sender, base_path);
        let consumers = self.start_receivers(session, receiver, dsettings);

        let (collection_result, _) = join!(task_stream, consumers);
        collection_result
    }

    async fn start_receivers(
        &self,
        session: &Session,
        receiver: Receiver<Task>,
        dsettings: &DownloadSettings,
    ) {
        let mut futures = FuturesUnordered::new();
        for _ in 0..10 {
            futures.push(self.start_consumer(session.clone(), receiver.clone(), dsettings));
        }

        while let Some(_) = futures.next().await {}
    }

    async fn start_consumer(
        &self,
        session: Session,
        receiver: Receiver<Task>,
        dsettings: &DownloadSettings,
    ) {
        while let Ok(task) = receiver.recv().await {
            let download_args = self
                .download_args
                .as_ref()
                .unwrap_or(&dsettings.download_args);

            if task.path.is_absolute() {
                panic!("Absolute paths are not allowed")
            }

            // let path = dsettings.

        }
    }
}

#[async_trait]
impl NodeTypeExt for Site {
    async fn path_segment(&self, session: &Session) -> Result<&Path, TemplateError> {
        self.module.folder_name(session).await
    }
}

#[derive(Config, Clone, Serialize, Debug)]
pub struct DownloadArgs {
    pub allowed_extensions: Vec<String>,
    pub forbidden_extensions: Vec<String>,
}

#[derive(Config, Clone, Serialize, Debug)]
pub struct SiteStorage {}

impl SiteStorage {}

#[derive(Config, Clone, Serialize)]
pub struct MetaData {}
