use crate::errors::TemplateError;
use crate::session::Session;
use crate::site_modules::Module;
use crate::task::Task;
use async_recursion::async_recursion;
use async_trait::async_trait;
use config::{Config, ConfigEnum};
use config_derive::Config;
use enum_dispatch::enum_dispatch;
use futures::{
    future::{Fuse, FusedFuture, FutureExt},
    pin_mut, select,
    stream::{FusedStream, Stream, StreamExt},
};
use lazy_static::lazy_static;
use regex::Regex;
use sha1::{Digest, Sha1};
use std::collections::HashMap;
use std::path::Path;
use std::path::PathBuf;
use tokio::fs;

use futures::future::try_join_all;

use crate::settings::DownloadSettings;
use futures::prelude::*;
use serde::Serialize;
use std::sync::Arc;
use std::sync::{Mutex, RwLock};
use tokio::try_join;

use crate::template::node_type::utils::{add_to_file_stem, extension_from_url};
use dashmap::mapref::entry::Entry;
use futures::stream::{FuturesUnordered, TryForEachConcurrent, TryStreamExt};
use futures::task::Poll;
use reqwest::header::HeaderMap;
use std::ffi::{OsStr, OsString};
use std::pin::Pin;
use std::task::Context;
use tokio::io::AsyncWriteExt;
use tokio::sync::mpsc::Receiver;
use url::Url;

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
    pub async fn path_segment(&self, session: &Session) -> Result<&Path, TemplateError> {
        self.module.folder_name(session).await
    }

    pub async fn run(
        self: Arc<Self>,
        session: Session,
        dsettings: Arc<DownloadSettings>,
        base_path: PathBuf,
    ) -> Result<(), TemplateError> {
        session.login(&self.module, &dsettings).await?;

        let (sender, receiver) = tokio::sync::mpsc::channel(1024);

        let task_stream = self
            .module
            .retrieve_urls(session.clone(), sender, base_path);

        let consumers = Arc::clone(&self).handle_receiver(session, receiver, dsettings);

        try_join!(task_stream, consumers)?;
        Ok(())
    }

    async fn handle_receiver(
        self: Arc<Self>,
        session: Session,
        mut receiver: Receiver<Task>,
        dsettings: Arc<DownloadSettings>,
    ) -> Result<(), TemplateError> {
        let mut futs = FuturesUnordered::new();
        loop {
            tokio::select! {
                Some(task) = receiver.recv() => {
                    let self_clone = Arc::clone(&self);
                    let handle = tokio::spawn(self_clone.consume_task(
                        session.clone(),
                        task,
                        Arc::clone(&dsettings),
                    ));
                    futs.push(handle);
                },
                Some(result) = futs.next() => {
                    let msg = result??;
                },
                else => break,
            }
        }
        Ok(())
    }

    async fn consume_task(
        self: Arc<Self>,
        session: Session,
        mut task: Task,
        dsettings: Arc<DownloadSettings>,
    ) -> Result<Msg, TemplateError> {
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
            return Ok(Msg::AlreadyExist);
        }

        let mut response = session.get(task.url).send().await?;
        if response.status() == reqwest::StatusCode::NOT_MODIFIED {
            return Ok(Msg::NotModified);
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
        Ok(Msg::AddedFile)
    }
}

#[derive(Debug)]
pub enum Msg {
    AddedFile,
    NotModified,
    AlreadyExist,
}

#[derive(Config, Serialize, Debug)]
pub struct DownloadArgs {
    pub allowed_extensions: Vec<String>,
    pub forbidden_extensions: Vec<String>,
}

#[derive(Config, Serialize, Debug)]
pub struct SiteStorage {
    #[config(ty = "HashMap", inner_ty = "struct")]
    pub files: dashmap::DashMap<PathBuf, FileData>,
}

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
