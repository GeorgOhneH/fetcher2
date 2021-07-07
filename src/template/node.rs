use crate::error::Result;
use crate::session::Session;
use async_recursion::async_recursion;
use config::{Config, ConfigEnum};
use config_derive::Config;
use sha1::Digest;
use std::path::PathBuf;

use futures::future::try_join_all;

use crate::settings::DownloadSettings;
use futures::prelude::*;
use serde::Serialize;
use std::sync::Arc;

use crate::template::node_type::NodeType;

#[derive(Config, Serialize, Debug)]
pub struct RootNode {
    #[config(inner_ty = "struct")]
    pub children: Vec<Node>,
}


impl RootNode {
    pub async fn run(&self, session: &Session, dsettings: Arc<DownloadSettings>) -> Result<()> {
        let futures: Vec<_> = self
            .children
            .iter()
            .map(|child| child.run(session, Arc::clone(&dsettings), PathBuf::new()))
            .collect();

        try_join_all(futures).await?;
        Ok(())
    }
}

#[derive(Config, Clone, Serialize, Debug)]
pub struct MetaData {}

#[derive(Config, Serialize, Debug)]
pub struct Node {
    #[config(ty = "enum")]
    pub ty: NodeType,
    #[config(inner_ty = "struct")]
    pub children: Vec<Node>,

    #[config(ty = "struct")]
    pub meta_data: MetaData,
}

impl Node {
    #[async_recursion]
    async fn run<'a>(
        &'a self,
        session: &'a Session,
        dsettings: Arc<DownloadSettings>,
        base_path: PathBuf,
    ) -> Result<()> {
        let segment = self.ty.path_segment(&session, &dsettings).await?;
        if segment.is_absolute() {
            panic!("segment is not allowed to be absolute")
        }
        let path = base_path.join(segment);

        let mut futures: Vec<_> = self
            .children
            .iter()
            .map(|child| child.run(session, Arc::clone(&dsettings), path.clone()))
            .collect();

        if let NodeType::Site(site) = &self.ty {
            let site_clone = site.clone();
            let handle = tokio::spawn(site_clone.run(session.clone(), dsettings, path));
            futures.push(Box::pin(async move { handle.await? }))
        };

        try_join_all(futures).await?;
        Ok(())
    }
}
