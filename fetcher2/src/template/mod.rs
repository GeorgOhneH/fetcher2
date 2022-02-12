use std::collections::HashSet;
use std::fmt::Debug;
use std::marker::PhantomData;
use std::path::{Path, PathBuf};
use std::pin::Pin;
use std::sync::Arc;
use tokio::sync::mpsc;
use tokio::sync::mpsc::Receiver;

use tokio::fs;
use tokio::io::AsyncWriteExt;

use crate::error::Result;
use crate::session::Session;
use crate::settings::DownloadSettings;
use crate::template::communication::{CommunicationExt, RawCommunicationExt};
pub use crate::template::node_type::{DownloadArgs, Extensions, Mode};
use crate::template::nodes::node::{NodeEvent, NodeEventKind, Status};
use crate::template::nodes::root::{RawRootNode, RootNode};

pub mod communication;
pub mod node_type;
pub mod nodes;

pub type NodeIndex = im::Vector<usize>;

#[derive(Debug)]
pub struct UnPrepared;
#[derive(Debug)]
pub struct Prepared;

#[derive(Debug)]
pub struct Template<T> {
    pub root: RootNode,
    pub save_path: Option<PathBuf>,
    _m: PhantomData<T>,
}

impl<T> Template<T> {
    pub async fn inform_of_cancel(&self) {
        self.root.inform_of_cancel().await
    }

    pub async fn save(&self) -> Result<()> {
        if let Some(save_path) = &self.save_path {
            let raw_root = self.root.clone().raw();
            let template_str = ron::ser::to_string(&raw_root)?;
            let mut f = fs::OpenOptions::new()
                .write(true)
                .truncate(true)
                .create(true)
                .open(save_path)
                .await?;
            f.write_all(template_str.as_bytes()).await?;

            f.shutdown().await?;
        }
        Ok(())
    }
}

impl Template<UnPrepared> {
    pub fn empty() -> Template<UnPrepared> {
        Self {
            root: RootNode::new(),
            save_path: None,
            _m: PhantomData,
        }
    }

    pub fn new(
        raw: RawRootNode,
        save_path: PathBuf,
    ) -> (Template<UnPrepared>, Receiver<NodeEvent>) {
        let (tx, rx) = mpsc::channel(1024);
        let root = raw.transform(tx);
        let template = Self {
            root,
            save_path: Some(save_path),
            _m: PhantomData,
        };
        (template, rx)
    }

    pub async fn load(path: &Path) -> Result<(Template<UnPrepared>, Receiver<NodeEvent>)> {
        let x = fs::read(path).await?;
        let raw_root: RawRootNode = ron::de::from_bytes(&x)?;
        Ok(Self::new(raw_root, path.to_owned()))
    }

    pub async fn prepare(
        mut self,
        dsettings: Arc<DownloadSettings>,
    ) -> std::result::Result<Template<Prepared>, Template<UnPrepared>> {
        let session = Session::new();
        let status = Pin::new(&mut self.root).prepare(&session, dsettings).await;
        if let Status::Success = status {
            Ok(Template::<Prepared> {
                root: self.root,
                save_path: self.save_path,
                _m: PhantomData,
            })
        } else {
            Err(self)
        }
    }
}

impl Template<Prepared> {
    pub async fn run_root(&self, dsettings: Arc<DownloadSettings>) {
        let session = Session::new();
        self.root.run(&session, dsettings, None).await
    }

    pub async fn run(&self, dsettings: Arc<DownloadSettings>, indexes: &HashSet<NodeIndex>) {
        let session = Session::new();
        self.root.run(&session, dsettings, Some(indexes)).await
    }
}

impl Default for Template<UnPrepared> {
    fn default() -> Self {
        Self::empty()
    }
}
