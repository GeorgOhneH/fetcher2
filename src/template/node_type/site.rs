use crate::error::{Result, TError, TErrorKind};
use crate::session::Session;
use crate::site_modules::Module;
use crate::site_modules::ModuleExt;
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
use std::collections::{HashMap, HashSet};
use std::path::Path;
use std::path::PathBuf;

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
use reqwest::header::{HeaderMap, HeaderValue};
use reqwest::{Request, RequestBuilder};
use std::ffi::{OsStr, OsString};
use std::pin::Pin;
use std::task::Context;
use std::time::Duration;
use tokio::io::AsyncWriteExt;
use tokio::sync::mpsc::Receiver;
use url::Url;
use druid::{Data, im, Widget, ExtEventSink};

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
    pub async fn path_segment(
        &self,
        session: &Session,
        dsettings: &DownloadSettings,
    ) -> Result<PathBuf> {
        self.module.folder_name(session, dsettings).await
    }

    pub async fn run(
        self: Arc<Self>,
        session: Session,
        dsettings: Arc<DownloadSettings>,
        base_path: PathBuf,
    ) -> Result<()> {
        self.module.real_login(&session, &dsettings).await?;

        let (sender, receiver) = tokio::sync::mpsc::channel(1024);

        let task_stream =
            self.module
                .retrieve_urls(session.clone(), sender, base_path, Arc::clone(&dsettings));

        let consumers = Arc::clone(&self).handle_receiver(session, receiver, dsettings);

        try_join!(task_stream, consumers)?;
        Ok(())
    }

    async fn handle_receiver(
        self: Arc<Self>,
        session: Session,
        mut receiver: Receiver<Task>,
        dsettings: Arc<DownloadSettings>,
    ) -> Result<()> {
        let mut futs = FuturesUnordered::new();
        let mut n_errors = 0;
        loop {
            tokio::select! {
                biased;

                Some(result) = futs.next() => {
                    let msg: Result<Msg> = result?;
                    if msg.is_ok() {} else { n_errors += 1 }
                    println!("{:?}, {}", msg, n_errors);
                },
                Some(task) = receiver.recv(), if futs.len() < 512 => {
                    let self_clone = Arc::clone(&self);
                    let handle = tokio::spawn(self_clone.consume_task(
                        session.clone(),
                        task,
                        Arc::clone(&dsettings),
                    ));
                    futs.push(handle);
                },
                else => break,
            }
        }
        Ok(())
    }

    async fn consume_task(
        self: Arc<Self>,
        session: Session,
        task: Task,
        dsettings: Arc<DownloadSettings>,
    ) -> Result<Msg> {
        let download_args = self
            .download_args
            .as_ref()
            .unwrap_or(&dsettings.download_args);

        let Task {
            path: mut task_path,
            url: task_url,
            headers: task_headers,
            basic_auth: task_basic_auth,
            bearer_auth: task_bearer_auth,
            checksum: task_checksum,
            has_extension: task_has_extension,
        } = task;

        assert!(task_path.is_relative());
        assert!(dsettings.save_path.is_absolute());

        if !task_has_extension {
            if let Some(extension) = extension_from_url(&session, &task_url).await? {
                let mut file_name = task_path.file_name().unwrap().to_os_string();
                file_name.push(".");
                file_name.push(extension);
                task_path.set_file_name(file_name);
            } else {
                // TODO: not panic
                panic!("efswwef")
            }
        }

        let final_path: PathBuf = dsettings.save_path.join(&task_path).into();
        let temp_path = add_to_file_stem(&final_path, "-temp");
        let old_path = add_to_file_stem(&final_path, "-old");

        if download_args
            .extensions
            .is_extension_forbidden(final_path.extension())
        {
            println!("{:?}", final_path);
            return Ok(Msg::ForbiddenExtension);
        }

        let is_task_checksum_same =
            self.storage
                .files
                .get(&final_path)
                .map_or(false, |file_data| {
                    if let (Some(cache), Some(current)) = (&file_data.task_checksum, &task_checksum)
                    {
                        cache == current
                    } else {
                        file_data.etag.is_none()
                    }
                });

        let action = if tokio::fs::metadata(&final_path).await.is_ok() {
            Action::Replace
        } else {
            Action::AddNew
        };

        if action == Action::Replace && is_task_checksum_same && !dsettings.force {
            return Ok(Msg::AlreadyExist);
        }

        let request = self.build_request(
            &session,
            task_url,
            task_headers,
            task_bearer_auth,
            task_basic_auth,
            &final_path,
        )?;

        let mut response = session.execute(request).await?.error_for_status()?;
        if response.status() == reqwest::StatusCode::NOT_MODIFIED {
            self.storage
                .files
                .get_mut(&final_path)
                .map(|mut file_data| file_data.task_checksum = task_checksum);
            return Ok(Msg::NotModified);
        }

        tokio::fs::create_dir_all(final_path.parent().unwrap()).await?;

        let mut hasher = Sha1::new();

        {
            let mut f = tokio::fs::OpenOptions::new()
                .write(true)
                .create(true)
                .truncate(true)
                .open(&temp_path)
                .await?;

            while let Some(chunk) =
                tokio::time::timeout(Duration::from_secs(10), response.chunk()).await??
            {
                hasher.update(&chunk);
                f.write_all(&chunk).await?
            }

            f.shutdown().await?;
        }

        let file_checksum = String::from_utf8_lossy(&hasher.finalize()[..]).into_owned();
        let etag = response
            .headers()
            .get("ETag")
            .map(|value| format_etag(value))
            .transpose()?;

        if action == Action::Replace {
            if let Some(mut file_data) = self.storage.files.get_mut(&final_path) {
                if file_data.file_checksum == file_checksum {
                    file_data.etag = etag;
                    return Ok(Msg::FileChecksumSame);
                }
            }
        }

        if action == Action::Replace && download_args.keep_old_files {
            std::fs::rename(&final_path, &old_path)?;
        }

        std::fs::rename(&temp_path, &final_path)?;

        match self.storage.files.entry(final_path) {
            Entry::Occupied(mut entry) => {
                let data = entry.get_mut();
                data.file_checksum = file_checksum;
                data.etag = etag;
                data.task_checksum = task_checksum;
            }
            Entry::Vacant(entry) => {
                let mut data = FileData::new(file_checksum);
                data.etag = etag;
                data.task_checksum = task_checksum;
                entry.insert(data);
            }
        }

        match action {
            Action::AddNew => Ok(Msg::AddedFile),
            Action::Replace => Ok(Msg::ReplacedFile),
        }
    }

    fn build_request(
        &self,
        session: &Session,
        task_url: Url,
        task_headers: Option<HeaderMap>,
        task_bearer_auth: Option<String>,
        task_basic_auth: Option<(String, Option<String>)>,
        final_path: &PathBuf,
    ) -> Result<Request> {
        let request_builder = session.get(task_url);

        let request_builder = match self.storage.files.get(final_path) {
            Some(file_data) => match file_data.etag.as_ref() {
                Some(etag) => request_builder.header("If-None-Match", etag),
                _ => request_builder,
            },
            _ => request_builder,
        };

        let request_builder = match task_headers {
            Some(headers) => request_builder.headers(headers),
            None => request_builder,
        };

        let request_builder = match task_bearer_auth {
            Some(token) => request_builder.bearer_auth(token),
            None => request_builder,
        };

        let request_builder = match task_basic_auth {
            Some((username, password)) => request_builder.basic_auth(username, password),
            None => request_builder,
        };

        let request = request_builder.build()?;
        Ok(request)
    }

    pub fn widget_data(&self) -> SiteData {
        SiteData {
            module: self.module.clone(),
            download_args: self.download_args.clone(),
        }
    }
}

fn format_etag(etag: &HeaderValue) -> Result<String> {
    Ok(etag
        .to_str()
        .map_err(|_| TErrorKind::ETagFormat)?
        .replace("-gzip", ""))
}

#[derive(Debug, PartialEq)]
pub enum Action {
    AddNew,
    Replace,
}

#[derive(Debug)]
pub enum Msg {
    AddedFile,
    ReplacedFile,
    NotModified,
    FileChecksumSame,
    AlreadyExist,
    ForbiddenExtension,
}

#[derive(Config, Serialize, Debug, Data, Clone)]
pub struct DownloadArgs {
    #[config(ty = "struct")]
    pub extensions: Extensions,

    #[config(default = true)]
    pub keep_old_files: bool,
}

#[derive(Config, Serialize, Debug, Clone, Data)]
pub struct Extensions {
    #[config(ty = "Vec")]
    pub inner: im::HashSet<String>,

    #[config(ty = "enum")]
    pub mode: Mode,
}

impl Extensions {
    pub fn is_extension_forbidden(&self, maybe_extension: Option<&OsStr>) -> bool {
        match maybe_extension {
            Some(extension) => match self.mode {
                Mode::Allowed => !self.inner.contains(&*extension.to_string_lossy()),
                Mode::Forbidden => self.inner.contains(&*extension.to_string_lossy()),
            },
            None => false,
        }
    }
}

#[derive(Config, Serialize, Debug, Clone, Data, PartialEq)]
pub enum Mode {
    Forbidden,
    Allowed,
}

#[derive(Config, Serialize, Debug)]
pub struct SiteStorage {
    #[config(ty = "HashMap", inner_ty = "struct")]
    pub files: dashmap::DashMap<PathBuf, FileData>,
}

#[derive(Config, Serialize, Debug)]
pub struct FileData {
    pub task_checksum: Option<String>,
    pub file_checksum: String,
    pub etag: Option<String>,
}

impl FileData {
    pub fn new(file_checksum: String) -> Self {
        Self {
            task_checksum: None,
            file_checksum,
            etag: None,
        }
    }
}

#[derive(Debug, Clone, Data)]
pub struct SiteData {
    pub module: Module,

    pub download_args: Option<DownloadArgs>,
}
