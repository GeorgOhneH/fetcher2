use std::path::Path;
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::Mutex;
use std::time::Duration;

use dashmap::mapref::entry::Entry;
use dashmap::DashMap;
use futures::prelude::*;
use futures::stream::FuturesUnordered;
use futures::stream::StreamExt;
use reqwest::header::{HeaderMap, HeaderValue};
use reqwest::Request;
use serde::{Deserialize, Serialize};
use sha1::{Digest, Sha1};
use tokio::io::AsyncReadExt;
use tokio::io::AsyncWriteExt;
use tokio::join;
use tokio::sync::mpsc::Receiver;
use tokio::task::JoinError;
use url::Url;

use config::traveller::Travel;

use crate::error::{Result, TError, TErrorKind};
use crate::session::Session;
use crate::settings::DownloadSettings;
use crate::site_modules::Module;
use crate::task::Task;
use crate::template::communication::RootNotifier;
use crate::template::node_type::utils::{add_to_file_stem, extension_from_url};
use crate::template::nodes::node::Status;
use crate::utils::spawn_drop;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Site {
    pub module: Module,

    pub storage: Arc<SiteStorage>,

    pub download_args: Option<DownloadArgs>,
}

impl Site {
    pub async fn path_segment(
        &self,
        session: &Session,
        dsettings: &DownloadSettings,
    ) -> Result<PathBuf> {
        self.module.login(session, dsettings).await?;
        self.module.folder_name(session, dsettings).await
    }

    pub async fn run(
        self: Arc<Self>,
        session: Session,
        dsettings: Arc<DownloadSettings>,
        base_path: PathBuf,
        tx: RootNotifier,
    ) {
        RunEventKind::wrapper(
            async {
                if LoginEventKind::wrapper(self.module.login(&session, &dsettings), &tx)
                    .await
                    .is_none()
                {
                    return;
                }

                let (sender, receiver) = tokio::sync::mpsc::channel(1024);

                let task_stream = UrlFetchEventKind::wrapper(
                    self.module
                        .fetch_urls(session.clone(), sender, Arc::clone(&dsettings)),
                    &tx,
                );

                let consumers = Arc::clone(&self).handle_receiver(
                    session,
                    receiver,
                    Arc::new(base_path),
                    dsettings,
                    tx.clone(),
                );

                join!(task_stream, consumers);
            },
            &tx,
        )
        .await
    }

    async fn handle_receiver(
        self: Arc<Self>,
        session: Session,
        mut receiver: Receiver<Task>,
        base_path: Arc<PathBuf>,
        dsettings: Arc<DownloadSettings>,
        tx: RootNotifier,
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
                        DownloadEventKind::wrapper(
                            self_clone.consume_task(
                                session.clone(),
                                task,
                                Arc::clone(&base_path),
                                Arc::clone(&dsettings),
                            ),
                            tx.clone(),
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
        base_path: Arc<PathBuf>,
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
        assert!(base_path.is_relative());
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

        let final_path = dsettings
            .save_path
            .join(base_path.as_ref())
            .join(&task_path);
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
                task_path,
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
            return Ok(TaskMsg::new(final_path, task_path, MsgKind::AlreadyExist));
        }

        let request = self.build_request(
            &session,
            task_url,
            task_headers,
            task_bearer_auth,
            task_basic_auth,
            &final_path,
            action,
        )?;

        let mut response = session.execute(request).await?.error_for_status()?;
        if response.status() == reqwest::StatusCode::NOT_MODIFIED {
            if let Some(mut file_data) = self.storage.files.get_mut(&final_path) {
                file_data.task_checksum = task_checksum
            }
            return Ok(TaskMsg::new(final_path, task_path, MsgKind::NotModified));
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
        // let current_file_checksum = Self::compute_file_checksum(temp_path.as_path()).await?;
        // assert_eq!(current_file_checksum, file_checksum); // TODO remove later
        let etag = response
            .headers()
            .get("ETag")
            .map(|value| format_etag(value))
            .transpose()?;

        if action == Action::Replace {
            if let Some(mut file_data) = self.storage.files.get_mut(&final_path) {
                if file_data.file_checksum == file_checksum {
                    file_data.etag = etag;
                    file_data.task_checksum = task_checksum;
                    return Ok(TaskMsg::new(
                        final_path,
                        task_path,
                        MsgKind::FileChecksumSame,
                    ));
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
                            return Ok(TaskMsg::new(
                                final_path,
                                task_path,
                                MsgKind::FileChecksumSame,
                            ));
                        }
                    }
                    Entry::Vacant(entry) => {
                        if current_file_checksum == file_checksum {
                            let data = FileData::new(file_checksum, etag, task_checksum);
                            entry.insert(data);
                            return Ok(TaskMsg::new(
                                final_path,
                                task_path,
                                MsgKind::FileChecksumSame,
                            ));
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
            Action::AddNew => Ok(TaskMsg::new(final_path, task_path, MsgKind::AddedFile)),
            Action::Replace => Ok(TaskMsg::new(
                final_path,
                task_path,
                MsgKind::ReplacedFile(old_path),
            )),
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
        final_path: &Path,
        action: Action,
    ) -> Result<Request> {
        let mut request_builder = session.get(task_url);

        if Action::AddNew != action {
            if let Some(file_data) = self.storage.files.get(final_path) {
                if let Some(etag) = file_data.etag.as_ref() {
                    request_builder = request_builder.header("If-None-Match", etag)
                }
            }
        }

        if let Some(headers) = task_headers {
            request_builder = request_builder.headers(headers)
        }

        if let Some(token) = task_bearer_auth {
            request_builder = request_builder.bearer_auth(token)
        }

        if let Some((username, password)) = task_basic_auth {
            request_builder = request_builder.basic_auth(username, password)
        }

        let request = request_builder.build()?;
        Ok(request)
    }
}

fn format_etag(etag: &HeaderValue) -> Result<String> {
    Ok(etag
        .to_str()
        .map_err(|_| TErrorKind::ETagFormat)?
        .replace("-gzip", ""))
}

#[derive(Debug, PartialEq, Copy, Clone)]
pub enum Action {
    AddNew,
    Replace,
}

#[cfg_attr(feature = "druid", derive(druid::Data))]
#[derive(Travel, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TaskMsg {
    pub kind: MsgKind,
    #[cfg_attr(feature = "druid", data(same_fn = "PartialEq::eq"))]
    pub full_path: PathBuf,
    #[cfg_attr(feature = "druid", data(same_fn = "PartialEq::eq"))]
    pub rel_path: PathBuf,
}

impl TaskMsg {
    pub fn new(full_path: PathBuf, rel_path: PathBuf, kind: MsgKind) -> Self {
        Self {
            full_path,
            rel_path,
            kind,
        }
    }
}

#[cfg_attr(feature = "druid", derive(druid::Data))]
#[derive(Travel, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum MsgKind {
    AddedFile,
    ReplacedFile(#[cfg_attr(feature = "druid", data(same_fn = "PartialEq::eq"))] PathBuf),
    NotModified,
    FileChecksumSame,
    AlreadyExist,
    ForbiddenExtension(Option<String>),
}

#[cfg_attr(feature = "druid", derive(druid::Data, druid::Lens))]
#[derive(Travel, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DownloadArgs {
    #[travel(name = "Extension Filter")]
    pub extensions: Extensions,

    #[travel(default = true, name = "Keep Old Files")]
    pub keep_old_files: bool,
}

#[cfg_attr(feature = "druid", derive(druid::Data, druid::Lens))]
#[derive(Travel, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Extensions {
    #[travel(default = Mode::Forbidden, name = "Mode")]
    pub mode: Mode,

    #[travel(name = "Extension")]
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

#[cfg_attr(feature = "druid", derive(druid::Data))]
#[derive(Travel, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Mode {
    Forbidden,
    Allowed,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct SiteStorage {
    pub files: dashmap::DashMap<PathBuf, FileData>,

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

impl Default for SiteStorage {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Travel, Debug, PartialEq, Serialize, Deserialize)]
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

#[derive(Debug)]
pub enum SiteEventKind {
    Run(RunEventKind),
    Login(LoginEventKind),
    UrlFetch(UrlFetchEventKind),
    Download(DownloadEventKind),
}

impl SiteEventKind {
    pub fn is_start(&self) -> bool {
        matches!(self, Self::Run(RunEventKind::Start))
    }
}

impl From<RunEventKind> for SiteEventKind {
    fn from(run_event: RunEventKind) -> Self {
        SiteEventKind::Run(run_event)
    }
}

impl From<LoginEventKind> for SiteEventKind {
    fn from(login_status: LoginEventKind) -> Self {
        SiteEventKind::Login(login_status)
    }
}

impl From<UrlFetchEventKind> for SiteEventKind {
    fn from(fetch_status: UrlFetchEventKind) -> Self {
        SiteEventKind::UrlFetch(fetch_status)
    }
}

impl From<DownloadEventKind> for SiteEventKind {
    fn from(download_status: DownloadEventKind) -> Self {
        SiteEventKind::Download(download_status)
    }
}

#[derive(Debug)]
pub enum RunEventKind {
    Start,
    Finish,
}

impl RunEventKind {
    pub async fn wrapper<T>(inner_fn: impl Future<Output = T>, tx: &RootNotifier) -> T {
        tx.notify(Self::Start).await;
        let r = inner_fn.await;
        tx.notify(Self::Finish).await;
        r
    }
}

#[derive(Debug)]
pub enum LoginEventKind {
    Start,
    Finish,
    Err(TError),
}

impl LoginEventKind {
    pub async fn wrapper<T>(
        inner_fn: impl Future<Output = Result<T>>,
        tx: &RootNotifier,
    ) -> Option<T> {
        tx.notify(Self::Start).await;
        match inner_fn.await {
            Ok(r) => {
                tx.notify(Self::Finish).await;
                Some(r)
            }
            Err(err) => {
                tx.notify(Self::Err(err)).await;
                None
            }
        }
    }
}

#[derive(Debug)]
pub enum UrlFetchEventKind {
    Start,
    Finish,
    Err(TError),
}

impl UrlFetchEventKind {
    pub async fn wrapper<T>(
        inner_fn: impl Future<Output = Result<T>>,
        tx: &RootNotifier,
    ) -> Option<T> {
        tx.notify(Self::Start).await;
        match inner_fn.await {
            Ok(r) => {
                tx.notify(Self::Finish).await;
                Some(r)
            }
            Err(err) => {
                tx.notify(Self::Err(err)).await;
                None
            }
        }
    }
}

#[derive(Debug)]
pub enum DownloadEventKind {
    Start,
    Finish(TaskMsg),
    Err(TError),
}

impl DownloadEventKind {
    pub async fn wrapper(
        inner_fn: impl Future<Output = Result<TaskMsg>>,
        tx: RootNotifier,
        site: Arc<Site>,
    ) -> Status {
        tx.notify(Self::Start).await;
        match inner_fn.await {
            Ok(msg) => {
                site.storage.history.lock().unwrap().push(msg.clone());
                dbg!("{:?}", &msg);
                tx.notify(Self::Finish(msg)).await;
                Status::Success
            }
            Err(err) => {
                tx.notify(Self::Err(err)).await;
                Status::Failure
            }
        }
    }
}
