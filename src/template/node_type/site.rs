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

use crate::template::communication::WidgetCommunication;
use crate::template::node_type::utils::{add_to_file_stem, extension_from_url};
use crate::template::nodes::node_data::CurrentState;
use crate::utils::spawn_drop;
use dashmap::mapref::entry::Entry;
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
use url::Url;

#[derive(Config, Serialize, Debug)]
pub struct Site {
    #[config(ty = "Enum")]
    pub module: Module,

    #[config(ty = "Struct")]
    pub storage: SiteStorage,

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
        comm: WidgetCommunication,
    ) -> Result<()> {
        comm.send_event(RunEvent::Start)?;
        comm.send_event(LoginEvent::Start)?;
        match self.module.real_login(&session, &dsettings).await {
            Ok(()) => comm.send_event(LoginEvent::Finish)?,
            Err(err) => {
                comm.send_event(LoginEvent::Err(err))?;
                return Ok(());
            }
        };

        let (sender, receiver) = tokio::sync::mpsc::channel(1024);

        let task_stream = self.module.real_fetch_urls(
            session.clone(),
            sender,
            base_path,
            Arc::clone(&dsettings),
            comm.clone(),
        );

        let consumers =
            Arc::clone(&self).handle_receiver(session, receiver, dsettings, comm.clone());

        try_join!(task_stream, consumers)?;
        comm.send_event(RunEvent::Finish)?;
        Ok(())
    }

    async fn handle_receiver(
        self: Arc<Self>,
        session: Session,
        mut receiver: Receiver<Task>,
        dsettings: Arc<DownloadSettings>,
        comm: WidgetCommunication,
    ) -> Result<()> {
        let mut futs = FuturesUnordered::new();
        loop {
            tokio::select! {
                biased;

                Some(msg) = futs.next() => {
                    self.handle_msg(msg?, &comm).await?
                },
                Some(task) = receiver.recv(), if futs.len() < 512 => {
                    let self_clone = Arc::clone(&self);
                    let handle = spawn_drop(self_clone.consume_task(
                        session.clone(),
                        task,
                        Arc::clone(&dsettings),
                    ));
                    futs.push(handle);
                    comm.send_event(DownloadEvent::Start)?;
                },
                else => break,
            }
        }
        Ok(())
    }

    async fn handle_msg(&self, msg: Result<Msg>, comm: &WidgetCommunication) -> Result<()> {
        match msg {
            Ok(msg) => {
                println!("{:?}", msg);
                self.storage.history.lock().unwrap().push(msg.clone());
                comm.send_event(DownloadEvent::Finish(msg))?;
            }
            Err(err) => {
                comm.send_event(DownloadEvent::Err(err))?;
            }
        }

        Ok(())
    }

    // TODO: make sure it's fine to call this function twice with same arguments
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

        let extension = final_path
            .extension()
            .map(|os_str| os_str.to_string_lossy().to_string());
        if download_args.extensions.is_extension_forbidden(&extension) {
            println!("{:?}", final_path);
            return Ok(Msg::new(final_path, MsgKind::ForbiddenExtension(extension)));
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
            return Ok(Msg::new(final_path, MsgKind::AlreadyExist));
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
            return Ok(Msg::new(final_path, MsgKind::NotModified));
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
                    return Ok(Msg::new(final_path, MsgKind::FileChecksumSame));
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
                let mut data = FileData::new(file_checksum);
                data.etag = etag;
                data.task_checksum = task_checksum;
                entry.insert(data);
            }
        }

        match action {
            Action::AddNew => Ok(Msg::new(final_path, MsgKind::AddedFile)),
            Action::Replace => Ok(Msg::new(final_path, MsgKind::ReplacedFile(old_path))),
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
            history: self.storage.history.lock().unwrap().clone().into(),
            state: SiteState::new(),
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
pub enum SiteEvent {
    Run(RunEvent),
    Login(LoginEvent),
    UrlFetch(UrlFetchEvent),
    Download(DownloadEvent),
}

impl From<RunEvent> for SiteEvent {
    fn from(run_event: RunEvent) -> Self {
        SiteEvent::Run(run_event)
    }
}

impl From<LoginEvent> for SiteEvent {
    fn from(login_status: LoginEvent) -> Self {
        SiteEvent::Login(login_status)
    }
}

impl From<UrlFetchEvent> for SiteEvent {
    fn from(fetch_status: UrlFetchEvent) -> Self {
        SiteEvent::UrlFetch(fetch_status)
    }
}

impl From<DownloadEvent> for SiteEvent {
    fn from(download_status: DownloadEvent) -> Self {
        SiteEvent::Download(download_status)
    }
}

#[derive(Debug)]
pub enum RunEvent {
    Start,
    Finish,
}

#[derive(Debug)]
pub enum LoginEvent {
    Start,
    Finish,
    Err(TError),
}

#[derive(Debug)]
pub enum UrlFetchEvent {
    Start,
    Finish,
    Err(TError),
}

#[derive(Debug)]
pub enum DownloadEvent {
    Start,
    Finish(Msg),
    Err(TError),
}

#[derive(Config, Serialize, Debug, Clone, Data)]
pub struct Msg {
    #[config(ty = "Enum")]
    pub kind: MsgKind,
    #[data(same_fn = "PartialEq::eq")]
    pub path: PathBuf,
}

impl Msg {
    pub fn new(path: PathBuf, kind: MsgKind) -> Self {
        Self { path, kind }
    }
}

#[derive(Config, Serialize, Debug, Clone, Data, PartialEq)]
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
    #[config(ty = "Vec", name = "Extension")]
    pub inner: im::HashSet<String>,

    #[config(ty = "Enum", default = "Forbidden", name = "Mode")]
    pub mode: Mode,
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

#[derive(Config, Serialize, Debug, Clone, Data, PartialEq)]
pub enum Mode {
    Forbidden,
    Allowed,
}

#[derive(Config, Serialize, Debug)]
pub struct SiteStorage {
    #[config(ty = "HashMap<_, Struct>")]
    pub files: dashmap::DashMap<PathBuf, FileData>,

    #[config(ty = "_<_<Struct>>")]
    pub history: Mutex<Vec<Msg>>,
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

    pub history: Vector<Msg>,

    pub state: SiteState,
}

impl SiteData {
    pub fn name(&self) -> String {
        self.module.name()
    }

    pub fn added_replaced(&self) -> (usize, usize) {
        (
            self.state.download.new_added,
            self.state.download.new_replaced,
        )
    }
}

#[derive(Debug, Clone, Data)]
pub struct SiteState {
    pub run: usize,
    pub login: LoginState,
    pub fetch: FetchState,
    pub download: DownloadState,
}

impl SiteState {
    pub fn new() -> Self {
        Self {
            run: 0,
            login: LoginState::new(),
            fetch: FetchState::new(),
            download: DownloadState::new(),
        }
    }

    pub fn update(&mut self, event: SiteEvent, history: &mut Vector<Msg>) {
        match event {
            SiteEvent::Run(run_event) => match run_event {
                RunEvent::Start => self.run += 1,
                RunEvent::Finish => self.run -= 1,
            },
            SiteEvent::Login(login_event) => self.login.update(login_event),
            SiteEvent::UrlFetch(fetch_event) => self.fetch.update(fetch_event),
            SiteEvent::Download(down_event) => self.download.update(down_event, history),
        }
    }
}

#[derive(Debug, Clone, Data)]
pub struct LoginState {
    pub count: usize,
    pub errs: Vector<Arc<TError>>,
}

impl LoginState {
    pub fn new() -> Self {
        Self {
            count: 0,
            errs: Vector::new(),
        }
    }

    pub fn update(&mut self, event: LoginEvent) {
        match event {
            LoginEvent::Start => self.count += 1,
            LoginEvent::Finish => self.count -= 1,
            LoginEvent::Err(err) => {
                self.errs.push_back(Arc::new(err));
                self.count -= 1
            }
        }
    }

    pub fn current_state(&self) -> CurrentState {
        if self.count != 0 {
            CurrentState::Active
        } else if self.errs.len() != 0 {
            CurrentState::Error
        } else {
            CurrentState::Idle
        }
    }
}

#[derive(Debug, Clone, Data)]
pub struct FetchState {
    pub count: usize,
    pub errs: Vector<Arc<TError>>,
}

impl FetchState {
    pub fn new() -> Self {
        Self {
            count: 0,
            errs: Vector::new(),
        }
    }

    pub fn update(&mut self, event: UrlFetchEvent) {
        match event {
            UrlFetchEvent::Start => self.count += 1,
            UrlFetchEvent::Finish => self.count -= 1,
            UrlFetchEvent::Err(err) => {
                self.errs.push_back(Arc::new(err));
                self.count -= 1
            }
        }
    }

    pub fn current_state(&self) -> CurrentState {
        if self.count != 0 {
            CurrentState::Active
        } else if self.errs.len() != 0 {
            CurrentState::Error
        } else {
            CurrentState::Idle
        }
    }
}

#[derive(Debug, Clone, Data)]
pub struct DownloadState {
    pub count: usize,
    pub total: usize,
    pub new_added: usize,
    pub new_replaced: usize,
    pub errs: Vector<Arc<TError>>,
}

impl DownloadState {
    pub fn new() -> Self {
        Self {
            count: 0,
            total: 0,
            new_added: 0,
            new_replaced: 0,
            errs: Vector::new(),
        }
    }

    pub fn update(&mut self, event: DownloadEvent, history: &mut Vector<Msg>) {
        match event {
            DownloadEvent::Start => {
                self.count += 1;
                self.total += 1
            }
            DownloadEvent::Finish(msg) => {
                match &msg {
                    Msg {
                        kind: MsgKind::AddedFile,
                        ..
                    } => self.new_added += 1,
                    Msg {
                        kind: MsgKind::ReplacedFile(_),
                        ..
                    } => self.new_replaced += 1,
                    _ => {}
                }
                history.push_back(msg);
                self.count -= 1;
            }
            DownloadEvent::Err(err) => {
                self.errs.push_back(Arc::new(err));
                self.count -= 1
            }
        }
        if self.count == 0 {
            self.total = 0;
        }
    }

    pub fn state_string(&self) -> String {
        if self.count != 0 {
            format!("Processing {}/{}", self.total - self.count, self.total)
        } else if self.errs.len() != 0 {
            "Error while downloading files".to_string()
        } else {
            "Idle".to_string()
        }
    }
}
