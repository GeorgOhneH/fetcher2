use crate::errors::TemplateError;
use crate::session::Session;
use crate::site_modules::minimal::Minimal;
use crate::task::Task;
use async_trait::async_trait;
use config::Config;
use config_derive::Config;
use enum_dispatch::enum_dispatch;
use serde::Serialize;
use std::path::{Path, PathBuf};

use crate::settings::DownloadSettings;
use tokio::sync::mpsc::Sender;

#[derive(Config, Serialize, Debug)]
pub enum Module {
    #[config(ty = "struct")]
    Minimal(Minimal),
}

impl Module {
    pub async fn retrieve_urls(
        &self,
        session: Session,
        sender: Sender<Task>,
        base_path: PathBuf,
    ) -> Result<(), TemplateError> {
        match &self {
            Module::Minimal(minimal) => minimal.retrieve_urls(session, sender, base_path).await,
        }
    }

    pub async fn login(
        &self,
        session: &Session,
        dsettings: &DownloadSettings,
    ) -> Result<(), TemplateError> {
        match &self {
            Module::Minimal(minimal) => minimal.login(session, dsettings).await,
        }
    }

    pub fn website_url(&self) -> String {
        match &self {
            Module::Minimal(minimal) => minimal.website_url(),
        }
    }

    pub async fn folder_name(&self, session: &Session) -> Result<&Path, TemplateError> {
        match &self {
            Module::Minimal(minimal) => minimal.folder_name(session).await,
        }
    }
}
