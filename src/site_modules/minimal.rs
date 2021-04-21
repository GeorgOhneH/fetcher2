use crate::errors::Error;
use crate::task::{Task};
use crate::session::Session;
use crate::site_modules::ModuleExt;
use async_trait::async_trait;
use config::Config;
use config_derive::Config;
use serde::Serialize;
use async_std::path::{PathBuf, Path};
use async_std::channel::Sender;
use lazy_static::lazy_static;
use tokio::time::Duration;

#[derive(Config, Clone, Serialize)]
pub struct Minimal {
    pub parameters: Option<String>,
}


impl Minimal {
}


#[async_trait]
impl ModuleExt for Minimal {
    async fn retrieve_urls(
        &self,
        session: &Session,
        queue: Sender<Task>,
        base_path: PathBuf,
    ) -> Result<(), Error> {
        println!("Retirevinbg Urls");
        Ok(())
    }

    async fn login(&self, session: &Session) -> Result<(), Error> {
        println!("LOGIN MINIMAL");
        tokio::time::sleep(Duration::from_secs(3)).await;
        Ok(())
    }

    fn website_url(&self) -> String {
        "todo!()".to_owned()
    }

    async fn folder_name(
        &self,
        session: &Session,
    ) -> Result<&Path, Error> {
        println!("Folder Name");
        Ok(Path::new("efgeuif"))
    }
}
