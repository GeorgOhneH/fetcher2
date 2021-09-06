use crate::error::{Result, TErrorKind};
use crate::session::Session;
use crate::site_modules::minimal::Minimal;
use crate::task::Task;
use async_trait::async_trait;
use config::Config;
use config_derive::Config;
use enum_dispatch::enum_dispatch;
use fetcher2_macro::{login_locks, LoginLock};
use serde::Serialize;
use std::path::PathBuf;
use tokio::sync::{Mutex, MutexGuard};

use crate::settings::DownloadSettings;
use crate::site_modules::polybox::Polybox;
use crate::template::communication::Communication;
use crate::template::node_type::site::UrlFetchEvent;
use druid::Data;
use std::string::ToString;
use std::sync::Arc;
use strum_macros::Display;
use tokio::sync::mpsc::Sender;

#[enum_dispatch(ModuleExt)]
#[login_locks]
#[derive(Config, Serialize, Debug, LoginLock, Data, Clone, Display)]
pub enum Module {
    #[config(ty = "Struct")]
    Minimal(Minimal),

    #[config(ty = "Struct")]
    Polybox(Polybox),
}

impl Module {
    pub async fn real_login(&self, session: &Session, dsettings: &DownloadSettings) -> Result<()> {
        let mut lock = self.get_lock(&session.login_mutex).await;
        match &*lock {
            LoginState::Success => Ok(()),
            LoginState::Failure => Err(TErrorKind::PreviousLoginError.into()),
            LoginState::Uninitiated => {
                let r = self.login(session, dsettings).await;
                *lock = if r.is_ok() {
                    LoginState::Success
                } else {
                    LoginState::Failure
                };
                r
            }
        }
    }

    pub async fn real_fetch_urls(
        &self,
        session: Session,
        sender: Sender<Task>,
        base_path: PathBuf,
        dsettings: Arc<DownloadSettings>,
        comm: Communication,
    ) {
        comm.send_event(UrlFetchEvent::Start);
        match self.fetch_urls(session, sender, base_path, dsettings).await {
            Ok(()) => comm.send_event(UrlFetchEvent::Finish),
            Err(err) => comm.send_event(UrlFetchEvent::Err(err)),
        }
    }

    pub fn name(&self) -> String {
        self.to_string()
    }
}

#[login_locks]
#[derive(Default)]
pub struct LoginLocks {
    pub aai_login: Mutex<LoginState>,
}

pub enum LoginState {
    Success,
    Failure,
    Uninitiated,
}

impl LoginState {
    pub fn new() -> Self {
        Self::Uninitiated
    }
}

impl Default for LoginState {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
#[enum_dispatch]
pub trait ModuleExt {
    async fn fetch_urls(
        &self,
        session: Session,
        sender: Sender<Task>,
        base_path: PathBuf,
        dsettings: Arc<DownloadSettings>,
    ) -> Result<()>;

    async fn login(&self, _session: &Session, _dsettings: &DownloadSettings) -> Result<()> {
        Ok(())
    }

    fn website_url(&self, dsettings: &DownloadSettings) -> String;

    async fn folder_name(&self, session: &Session, dsettings: &DownloadSettings)
        -> Result<PathBuf>;
}
