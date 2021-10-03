use std::collections::HashSet;
use std::fmt::Debug;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use tokio::fs;
use tokio::io::AsyncWriteExt;

use crate::error::Result;
use crate::session::Session;
use crate::settings::DownloadSettings;
use crate::template::communication::{CommunicationExt, RawCommunicationExt};
pub use crate::template::node_type::{DownloadArgs, Extensions, Mode};
use crate::template::nodes::node::Status;
use crate::template::nodes::root::{RawRootNode, RootNode};

pub mod communication;
pub mod node_type;
pub mod nodes;

pub type NodeIndex = Vec<usize>;

#[derive(Debug)]
pub struct Template<T> {
    pub root: RootNode<T>,
    pub save_path: Option<PathBuf>,
    is_prepared: bool,
}

impl<T: CommunicationExt> Template<T> {
    pub fn empty() -> Self {
        Self {
            root: RootNode::new(),
            is_prepared: false,
            save_path: None,
        }
    }

    pub fn new(root: RootNode<T>, save_path: PathBuf) -> Self {
        Self {
            root,
            is_prepared: false,
            save_path: Some(save_path),
        }
    }

    pub async fn load(path: &Path, comm: impl RawCommunicationExt<T>) -> Result<Self> {
        let x = fs::read(path).await?;
        let raw_root: RawRootNode = ron::de::from_bytes(&x)?;
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
