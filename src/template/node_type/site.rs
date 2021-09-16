use crate::error::{Result, TError, TErrorKind};
use crate::session::Session;
use crate::site_modules::Module;
use crate::site_modules::ModuleExt;
use crate::task::Task;
use async_recursion::async_recursion;
use async_trait::async_trait;
use config::{Config, ConfigEnum};
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
use tokio::io::AsyncReadExt;
use tokio::join;

use crate::template::communication::Communication;
use crate::template::node_type::site_data::{
    DownloadEvent, LoginEvent, RunEvent, SiteData, SiteState, UrlFetchEvent,
};
use crate::template::node_type::site_edit_data::SiteEditData;
use crate::template::node_type::utils::{add_to_file_stem, extension_from_url};
use crate::template::nodes::node::Status;
use crate::template::nodes::node_data::CurrentState;
use crate::utils::spawn_drop;
use dashmap::mapref::entry::Entry;
use dashmap::DashMap;
use druid::im::Vector;
use druid::{im, Data, ExtEventSink, Widget};
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
use tokio::task::JoinError;
use url::Url;

#[derive(Config, Serialize, Debug)]
pub struct Site {
    #[config(ty = "Enum")]
    pub module: Module,

    #[config(ty = "_<Struct>")]
    pub storage: Arc<SiteStorage>,

    #[config(ty = "_<Struct>")]
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
        comm: Communication,
    ) {
        RunEvent::wrapper(
            async {
                if LoginEvent::wrapper(self.module.login(&session, &dsettings), &comm)
                    .await
                    .is_none()
                {
                    return;
                }

                let (sender, receiver) = tokio::sync::mpsc::channel(1024);

                let task_stream = UrlFetchEvent::wrapper(
                    self.module.fetch_urls(
                        session.clone(),
                        sender,
                        base_path,
                        Arc::clone(&dsettings),
                    ),
                    &comm,
                );

                let consumers =
                    Arc::clone(&self).handle_receiver(session, receiver, dsettings, comm.clone());

                join!(task_stream, consumers);
            },
            &comm,
        )
        .await
    }

    async fn handle_receiver(
        self: Arc<Self>,
        session: Session,
        mut receiver: Receiver<Task>,
        dsettings: Arc<DownloadSettings>,
        comm: Communication,
    ) {
        let mut futs = FuturesUnordered::new();
        loop {
            tokio::select! {
                biased;

                Some(handle) = futs.next() => {
                    let handel: std::result::Result<_, JoinError> = handle;
                    handel.unwrap();
                },
                Some(task) = receiver.recv(), if futs.len() < 512 => {
                    let self_clone = Arc::clone(&self);
                    let handle = spawn_drop(
                        DownloadEvent::wrapper(
                            self_clone.consume_task(
                                session.clone(),
                                task,
                                Arc::clone(&dsettings),
                            ),
                            comm.clone(),
                            Arc::clone(&self),
                        )
                    );
                    futs.push(handle);
                },
                else => break,
            }
        }
    }

    // TODO: make sure it's fine to call this function twice with same arguments
    async fn consume_task(
        self: Arc<Self>,
        session: Session,
        task: Task,
        dsettings: Arc<DownloadSettings>,
    ) -> Result<TaskMsg> {
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
        // TODO use real temp file
        let temp_path = add_to_file_stem(&final_path, "-temp");
        let old_path = add_to_file_stem(&final_path, "-old");

        let extension = final_path
            .extension()
            .map(|os_str| os_str.to_string_lossy().to_string());
        if download_args.extensions.is_extension_forbidden(&extension) {
            println!("{:?}", final_path);
            return Ok(TaskMsg::new(
                final_path,
                MsgKind::ForbiddenExtension(extension),
            ));
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
            return Ok(TaskMsg::new(final_path, MsgKind::AlreadyExist));
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
            return Ok(TaskMsg::new(final_path, MsgKind::NotModified));
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
        let current_file_checksum = Self::compute_file_checksum(temp_path.as_path()).await?;
        assert_eq!(current_file_checksum, file_checksum); // TODO remove later
        let etag = response
            .headers()
            .get("ETag")
            .map(|value| format_etag(value))
            .transpose()?;

        if action == Action::Replace {
            if let Some(mut file_data) = self.storage.files.get_mut(&final_path) {
                dbg!(&file_data.file_checksum, &file_checksum);
                if file_data.file_checksum == file_checksum {
                    file_data.etag = etag;
                    file_data.task_checksum = task_checksum;
                    return Ok(TaskMsg::new(final_path, MsgKind::FileChecksumSame));
                }
            } else {
                let current_file_checksum =
                    Self::compute_file_checksum(final_path.as_path()).await?;
                match self.storage.files.entry(final_path.clone()) {
                    Entry::Occupied(mut entry) => {
                        let file_data = entry.get_mut();
                        if file_data.file_checksum == file_checksum {
                            file_data.etag = etag;
                            file_data.task_checksum = task_checksum;
                            return Ok(TaskMsg::new(final_path, MsgKind::FileChecksumSame));
                        }
                    }
                    Entry::Vacant(entry) => {
                        if current_file_checksum == file_checksum {
                            let data = FileData::new(file_checksum, etag, task_checksum);
                            entry.insert(data);
                            return Ok(TaskMsg::new(final_path, MsgKind::FileChecksumSame));
                        }
                    }
                }
            }
        }

        if action == Action::Replace && download_args.keep_old_files {
            std::fs::rename(&final_path, &old_path)?;
        }

        std::fs::rename(&temp_path, &final_path)?;

        match self.storage.files.entry(final_path.clone()) {
            Entry::Occupied(mut entry) => {
                let data = entry.get_mut();
                data.file_checksum = file_checksum;
                data.etag = etag;
                data.task_checksum = task_checksum;
            }
            Entry::Vacant(entry) => {
                let data = FileData::new(file_checksum, etag, task_checksum);
                entry.insert(data);
            }
        }

        match action {
            Action::AddNew => Ok(TaskMsg::new(final_path, MsgKind::AddedFile)),
            Action::Replace => Ok(TaskMsg::new(final_path, MsgKind::ReplacedFile(old_path))),
        }
    }

    async fn compute_file_checksum(path: &Path) -> Result<String> {
        let mut hasher = Sha1::new();
        let mut f = tokio::fs::OpenOptions::new().read(true).open(path).await?;
        let mut buffer = [0u8; 64 * 1024];
        loop {
            let chunk_size = f.read(&mut buffer).await?;
            if chunk_size == 0 {
                break;
            }
            hasher.update(&buffer[..chunk_size]);
        }
        Ok(String::from_utf8_lossy(&hasher.finalize()[..]).into_owned())
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
            history: self.storage.history.lock().unwrap().clone().into(),
            state: SiteState::new(),
        }
    }

    pub fn widget_edit_data(&self) -> SiteEditData {
        SiteEditData {
            module: self.module.clone(),
            download_args: self.download_args.clone(),
            storage: Some(Arc::clone(&self.storage)),
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

#[derive(Config, Serialize, Debug, Clone, Data)]
pub struct TaskMsg {
    #[config(ty = "Enum")]
    pub kind: MsgKind,
    #[data(same_fn = "PartialEq::eq")]
    pub path: PathBuf,
}

impl TaskMsg {
    pub fn new(path: PathBuf, kind: MsgKind) -> Self {
        Self { path, kind }
    }
}

#[derive(ConfigEnum, Serialize, Debug, Clone, Data, PartialEq)]
pub enum MsgKind {
    AddedFile,
    ReplacedFile(#[data(same_fn = "PartialEq::eq")] PathBuf),
    NotModified,
    FileChecksumSame,
    AlreadyExist,
    ForbiddenExtension(Option<String>),
}

#[derive(Config, Serialize, Debug, Data, Clone)]
pub struct DownloadArgs {
    #[config(ty = "Struct", name = "Extension Filter")]
    pub extensions: Extensions,

    #[config(default = true, name = "Keep Old Files")]
    pub keep_old_files: bool,
}

#[derive(Config, Serialize, Debug, Clone, Data)]
pub struct Extensions {
    #[config(ty = "Enum", default = "Forbidden", name = "Mode")]
    pub mode: Mode,

    #[config(ty = "Vec", name = "Extension")]
    pub inner: im::HashSet<String>,
}

impl Extensions {
    pub fn is_extension_forbidden(&self, maybe_extension: &Option<String>) -> bool {
        match maybe_extension {
            Some(extension) => match self.mode {
                Mode::Allowed => !self.inner.contains(extension),
                Mode::Forbidden => self.inner.contains(extension),
            },
            None => false,
        }
    }
}

#[derive(ConfigEnum, Serialize, Debug, Clone, Data, PartialEq)]
pub enum Mode {
    Forbidden,
    Allowed,
}

#[derive(Config, Serialize, Debug)]
pub struct SiteStorage {
    #[config(ty = "HashMap<_, Struct>")]
    pub files: dashmap::DashMap<PathBuf, FileData>,

    #[config(ty = "_<_<Struct>>")]
    pub history: Mutex<Vec<TaskMsg>>,
}

impl SiteStorage {
    pub fn new() -> Self {
        Self {
            files: DashMap::new(),
            history: Mutex::new(Vec::new()),
        }
    }
}

#[derive(Config, Serialize, Debug)]
pub struct FileData {
    pub task_checksum: Option<String>,
    pub file_checksum: String,
    pub etag: Option<String>,
}

impl FileData {
    pub fn new(file_checksum: String, etag: Option<String>, task_checksum: Option<String>) -> Self {
        Self {
            task_checksum,
            file_checksum,
            etag,
        }
    }
}
