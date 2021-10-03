use std::path::PathBuf;
use std::string::ToString;
use std::sync::Arc;

use async_trait::async_trait;
use druid::Data;
use enum_dispatch::enum_dispatch;
use strum_macros::Display;
use tokio::sync::mpsc::Sender;
use tokio::sync::{Mutex, MutexGuard};

use config::Config;
use config::ConfigEnum;
use fetcher2_macro::{login_locks, LoginLock};

use crate::error::{Result, TErrorKind};
use crate::session::Session;
use crate::settings::DownloadSettings;
use crate::site_modules::minimal::Minimal;
use crate::site_modules::moodle::Moodle;
use crate::site_modules::polybox::Polybox;
use crate::task::Task;

#[enum_dispatch(ModuleExt)]
#[login_locks]
#[derive(ConfigEnum, Debug, LoginLock, Data, Clone, Display, PartialEq)]
pub enum Module {
    #[config(ty = "struct")]
    Minimal(Minimal),

    #[config(ty = "struct")]
    Polybox(Polybox),

    #[config(ty = "struct")]
    Moodle(Moodle),
}

impl Module {
    pub async fn login(&self, session: &Session, dsettings: &DownloadSettings) -> Result<()> {
        let mut lock = self.get_lock(&session.login_mutex).await;
        match &*lock {
            LoginState::Success => Ok(()),
            LoginState::Failure => Err(TErrorKind::PreviousLoginError.into()),
            LoginState::Uninitiated => {
                let r = self.login_impl(session, dsettings).await;
                *lock = if r.is_ok() {
                    LoginState::Success
                } else {
                    LoginState::Failure
                };
                r
            }
        }
    }

    pub async fn fetch_urls(
        &self,
        session: Session,
        sender: Sender<Task>,
        dsettings: Arc<DownloadSettings>,
    ) -> Result<()> {
        self.fetch_urls_impl(session, sender, dsettings).await
    }

    pub async fn folder_name(
        &self,
        session: &Session,
        dsettings: &DownloadSettings,
    ) -> Result<PathBuf> {
        self.folder_name_impl(session, dsettings).await
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
    async fn fetch_urls_impl(
        &self,
        session: Session,
        sender: Sender<Task>,
        dsettings: Arc<DownloadSettings>,
    ) -> Result<()>;

    async fn login_impl(&self, _session: &Session, _dsettings: &DownloadSettings) -> Result<()> {
        Ok(())
    }

    fn website_url_impl(&self) -> String;

    async fn folder_name_impl(
        &self,
        session: &Session,
        dsettings: &DownloadSettings,
    ) -> Result<PathBuf>;
}
