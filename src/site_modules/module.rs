use crate::errors::Error;
use crate::task::Task;
use crate::session::Session;
use async_trait::async_trait;
use config::Config;
use async_std::path::{PathBuf, Path};
use enum_dispatch::enum_dispatch;
use crate::site_modules::minimal::Minimal;
use config_derive::Config;
use serde::Serialize;
use async_std::channel::Sender;

#[async_trait]
#[enum_dispatch]
pub trait ModuleExt {
    async fn retrieve_urls(
        &self,
        session: &Session,
        sender: Sender<Task>,
        base_path: PathBuf,
    ) -> Result<(), Error>;
    async fn login(&self, session: &Session) -> Result<(), Error>;
    fn website_url(&self) -> String;
    async fn folder_name(
        &self,
        session: &Session,
    ) -> Result<&Path, Error>;
}


#[enum_dispatch(ModuleExt)]
#[derive(Config, Clone, Serialize)]
pub enum Module {
    #[config(ty = "struct")]
    Minimal(Minimal)
}


