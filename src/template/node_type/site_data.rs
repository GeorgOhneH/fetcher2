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
use tokio::join;

use crate::template::communication::Communication;
use crate::template::node_type::site::{TaskMsg, MsgKind};
use crate::template::node_type::utils::{add_to_file_stem, extension_from_url};
use crate::template::node_type::Site;
use crate::template::nodes::node::Status;
use crate::template::nodes::node_data::CurrentState;
use crate::template::DownloadArgs;
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
use tokio::task::JoinError;
use url::Url;

#[derive(Debug, Clone, Data)]
pub struct SiteData {
    pub module: Module,

    pub download_args: Option<DownloadArgs>,

    pub history: Vector<TaskMsg>,

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

impl RunEvent {
    pub async fn wrapper<T>(inner_fn: impl Future<Output = T>, comm: &Communication) -> T {
        comm.send_event(Self::Start);
        let r = inner_fn.await;
        comm.send_event(Self::Finish);
        r
    }
}

#[derive(Debug)]
pub enum LoginEvent {
    Start,
    Finish,
    Err(TError),
}

impl LoginEvent {
    pub async fn wrapper<T>(
        inner_fn: impl Future<Output = Result<T>>,
        comm: &Communication,
    ) -> Option<T> {
        comm.send_event(Self::Start);
        match inner_fn.await {
            Ok(r) => {
                comm.send_event(Self::Finish);
                Some(r)
            }
            Err(err) => {
                comm.send_event(Self::Err(err));
                None
            }
        }
    }
}

#[derive(Debug)]
pub enum UrlFetchEvent {
    Start,
    Finish,
    Err(TError),
}

impl UrlFetchEvent {
    pub async fn wrapper<T>(
        inner_fn: impl Future<Output = Result<T>>,
        comm: &Communication,
    ) -> Option<T> {
        comm.send_event(Self::Start);
        match inner_fn.await {
            Ok(r) => {
                comm.send_event(Self::Finish);
                Some(r)
            }
            Err(err) => {
                comm.send_event(Self::Err(err));
                None
            }
        }
    }
}

#[derive(Debug)]
pub enum DownloadEvent {
    Start,
    Finish(TaskMsg),
    Err(TError),
}

impl DownloadEvent {
    pub async fn wrapper(
        inner_fn: impl Future<Output = Result<TaskMsg>>,
        comm: Communication,
        site: Arc<Site>,
    ) -> Status {
        comm.send_event(Self::Start);
        match inner_fn.await {
            Ok(msg) => {
                site.storage.history.lock().unwrap().push(msg.clone());
                println!("{:?}", msg);
                comm.send_event(Self::Finish(msg));
                Status::Success
            }
            Err(err) => {
                comm.send_event(Self::Err(err));
                Status::Failure
            }
        }
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

    pub fn update(&mut self, event: SiteEvent, history: &mut Vector<TaskMsg>) {
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

    pub fn update(&mut self, event: DownloadEvent, history: &mut Vector<TaskMsg>) {
        match event {
            DownloadEvent::Start => {
                self.count += 1;
                self.total += 1
            }
            DownloadEvent::Finish(msg) => {
                match &msg {
                    TaskMsg {
                        kind: MsgKind::AddedFile,
                        ..
                    } => self.new_added += 1,
                    TaskMsg {
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
