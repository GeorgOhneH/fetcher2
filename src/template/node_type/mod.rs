use crate::error::{Result, TError};
use crate::session::Session;
use crate::site_modules::Module;
use crate::task::Task;
use async_recursion::async_recursion;
use async_trait::async_trait;
use config::{Config, ConfigEnum};
use config_derive::Config;
use enum_dispatch::enum_dispatch;
use lazy_static::lazy_static;
use regex::Regex;
use sha1::{Digest, Sha1};
use std::collections::HashMap;
use std::path::Path;
use std::path::PathBuf;
use tokio::fs;

use futures::future::try_join_all;

use async_std::channel::{self, Receiver, Sender};
use futures::prelude::*;
use serde::Serialize;
use std::sync::Arc;
use std::sync::{Mutex, RwLock};
use tokio::try_join;

use dashmap::mapref::entry::Entry;
use futures::stream::{FuturesUnordered, TryStreamExt};
use reqwest::header::HeaderMap;
use std::ffi::{OsStr, OsString};
use tokio::io::AsyncWriteExt;
use url::Url;

mod folder;
mod site;
mod utils;

use crate::settings::DownloadSettings;
pub use crate::template::node_type::folder::Folder;
pub use crate::template::node_type::site::Mode;
pub use crate::template::node_type::site::Site;
pub use crate::template::node_type::site::SiteStorage;
pub use crate::template::node_type::site::{DownloadArgs, Extensions};

#[derive(Config, Serialize, Debug)]
pub enum NodeType {
    #[config(ty = "struct")]
    Folder(Folder),
    #[config(inner_ty = "struct")]
    Site(Arc<Site>),
}

impl NodeType {
    pub async fn path_segment(
        &self,
        session: &Session,
        dsettings: &DownloadSettings,
    ) -> Result<PathBuf> {
        match self {
            NodeType::Folder(folder) => folder.path_segment().await,
            NodeType::Site(site) => site.path_segment(session, dsettings).await,
        }
    }
}
