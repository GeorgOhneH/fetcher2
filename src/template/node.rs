use crate::errors::TemplateError;
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

use crate::settings::DownloadSettings;
use async_std::channel::{self, Receiver, Sender};
use tokio::try_join;
use futures::prelude::*;
use serde::Serialize;
use std::sync::Arc;
use std::sync::{Mutex, RwLock};

use dashmap::mapref::entry::Entry;
use futures::stream::{FuturesUnordered, TryStreamExt};
use reqwest::header::HeaderMap;
use std::ffi::{OsStr, OsString};
use tokio::io::AsyncWriteExt;
use url::Url;

#[derive(Config, Serialize, Debug)]
pub struct RootNode {
    #[config(inner_ty = "struct")]
    pub children: Vec<Node>,
}

impl RootNode {
    pub async fn run(
        &self,
        session: &Session,
        dsettings: Arc<DownloadSettings>,
    ) -> Result<(), TemplateError> {
        let futures: Vec<_> = self
            .children
            .iter()
            .map(|child| child.run(session, Arc::clone(&dsettings), PathBuf::new()))
            .collect();

        try_join_all(futures).await?;
        Ok(())
    }
}

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
    ) -> Result<(), TemplateError> {
        let segment = self.ty.path_segment(&session).await?;
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

#[derive(Config, Serialize, Debug)]
pub enum NodeType {
    #[config(ty = "struct")]
    Folder(Folder),
    #[config(inner_ty = "struct")]
    Site(Arc<Site>),
}

impl NodeType {
    async fn path_segment(&self, session: &Session) -> Result<&Path, TemplateError> {
        match self {
            NodeType::Folder(folder) => folder.path_segment(session).await,
            NodeType::Site(site) => site.path_segment(session).await,
        }
    }
}

#[derive(Config, Serialize, Debug)]
pub struct Folder {
    name: String,
}

impl Folder {
    async fn path_segment(&self, _session: &Session) -> Result<&Path, TemplateError> {
        Ok(Path::new(&self.name))
    }
}

#[derive(Config, Serialize, Debug)]
pub struct Site {
    #[config(ty = "enum")]
    pub module: Module,

    #[config(ty = "struct")]
    pub storage: SiteStorage,

    #[config(inner_ty = "struct")]
    pub download_args: Option<DownloadArgs>,
}

impl Site {
    async fn run(
        self: Arc<Self>,
        session: Session,
        dsettings: Arc<DownloadSettings>,
        base_path: PathBuf,
    ) -> Result<(), TemplateError> {
        session.login(&self.module, &dsettings).await?;

        let (sender, receiver) = async_std::channel::bounded(1024);

        let task_stream = self
            .module
            .retrieve_urls(session.clone(), sender, base_path);
        let consumers = Arc::clone(&self).start_receivers(session, receiver, dsettings);

        try_join!(task_stream, consumers)?;
        Ok(())
    }

    async fn start_receivers(
        self: Arc<Self>,
        session: Session,
        receiver: Receiver<Task>,
        dsettings: Arc<DownloadSettings>,
    ) -> Result<(), TemplateError> {
        let mut handles = vec![];
        for _ in 0..10 {
            let self_clone = Arc::clone(&self);
            handles.push(tokio::spawn(self_clone.start_consumer(
                session.clone(),
                receiver.clone(),
                Arc::clone(&dsettings),
            )));
        }

        for handle in handles {
            handle.await??;
        };
        Ok(())

    }

    async fn start_consumer(
        self: Arc<Self>,
        session: Session,
        receiver: Receiver<Task>,
        dsettings: Arc<DownloadSettings>,
    ) -> Result<(), TemplateError> {
        while let Ok(mut task) = receiver.recv().await {
            let download_args = self
                .download_args
                .as_ref()
                .unwrap_or(&dsettings.download_args);

            if task.path.is_absolute() {
                panic!("Absolute paths are not allowed")
            }

            if dsettings.save_path.is_relative() {
                panic!("Save Path must be absolute")
            }

            if !task.has_extension {
                if let Some(extension) = extension_from_url(&session, &task.url).await? {
                    let mut file_name = task.path.file_name().unwrap().to_os_string();
                    file_name.push(".");
                    file_name.push(extension);
                    task.path.set_file_name(file_name);
                } else {
                    // TODO: not panic
                    panic!("efswwef")
                }
            }

            let final_path: PathBuf = dsettings.save_path.join(&task.path).into();
            let temp_path = add_to_file_stem(&final_path, "-temp");
            let old_path = add_to_file_stem(&final_path, "-old");

            self.storage
                .files
                .get(&final_path)
                .map(|x| println!("{:#?}", *x));

            if fs::metadata(&final_path).await.is_ok() {
                return Ok(());
            }

            let mut response = session.get(task.url).send().await?;
            if response.status() == reqwest::StatusCode::NOT_MODIFIED {
                return Ok(());
            }

            fs::create_dir_all(final_path.parent().unwrap()).await?;

            let mut hasher = Sha1::new();

            {
                let mut f = fs::OpenOptions::new()
                    .write(true)
                    .create(true)
                    .truncate(true)
                    .open(temp_path)
                    .await?;
                while let Some(chunk) = response.chunk().await? {
                    hasher.update(&chunk);
                    f.write_all(&chunk).await?
                }

                f.shutdown().await?;
            }

            let file_checksum = String::from_utf8_lossy(&hasher.finalize()[..]).into_owned();

            match self.storage.files.entry(final_path) {
                Entry::Occupied(mut entry) => {
                    let data = entry.get_mut();
                    data.file_checksum = file_checksum;
                }
                Entry::Vacant(entry) => {
                    let data = FileData::new(file_checksum);
                    entry.insert(data);
                }
            }
        }
        Ok(())
    }
}

async fn extension_from_url(
    session: &Session,
    url: &Url,
) -> Result<Option<OsString>, TemplateError> {
    let response = session.get(url.clone()).send().await?;
    let headers = response.headers();

    if let Some(file_name) = filename_from_headers(headers) {
        Ok(PathBuf::from(file_name)
            .extension()
            .map(|os_str| os_str.to_os_string()))
    } else {
        let extension = headers
            .get_all("content-type")
            .iter()
            .filter_map(|x| x.to_str().ok())
            .flat_map(|mime_str| mime_guess::get_mime_extensions_str(mime_str).into_iter())
            .flatten()
            .next()
            .map(|x| OsString::from(x));
        Ok(extension)
    }
}

fn filename_from_headers(headers: &HeaderMap) -> Option<String> {
    lazy_static! {
        static ref FILENAME_RE: Regex = Regex::new("filename=\"(.+)\"").unwrap();
    }
    headers
        .get_all("content-disposition")
        .iter()
        .filter_map(|x| x.to_str().ok())
        .filter_map(|str| FILENAME_RE.captures(str))
        .map(|capture| capture[1].to_owned())
        .next()
}

fn add_to_file_stem(path: &PathBuf, name: &str) -> PathBuf {
    let mut file_name = path.file_stem().unwrap().to_os_string();
    file_name.push(name);

    if let Some(extension) = path.extension() {
        file_name.push(".");
        file_name.push(extension);
    };

    path.with_file_name(file_name)
}

impl Site {
    async fn path_segment(&self, session: &Session) -> Result<&Path, TemplateError> {
        self.module.folder_name(session).await
    }
}

#[derive(Config, Clone, Serialize, Debug)]
pub struct DownloadArgs {
    pub allowed_extensions: Vec<String>,
    pub forbidden_extensions: Vec<String>,
}

#[derive(Config, Serialize, Debug)]
pub struct SiteStorage {
    #[config(ty = "HashMap", inner_ty = "struct")]
    pub files: dashmap::DashMap<PathBuf, FileData>,
}

impl SiteStorage {}

#[derive(Config, Serialize, Debug)]
pub struct FileData {
    pub site_checksum: Option<String>,
    pub file_checksum: String,
}

impl FileData {
    pub fn new(file_checksum: String) -> Self {
        Self {
            site_checksum: None,
            file_checksum,
        }
    }
}

#[derive(Config, Clone, Serialize, Debug)]
pub struct MetaData {}
