use crate::errors::Error;
use crate::queue::Queue;
use crate::session::Session;
use async_trait::async_trait;
use config::Config;
use std::path::PathBuf;

#[async_trait]
pub trait Site{
    type Parameters: Config;
    async fn retrieve_urls(
        session: &mut Session,
        queue: &mut Queue,
        base_path: PathBuf,
        parameters: Self::Parameters,
    ) -> Result<(), Error>;
    async fn login(session: &mut Session, parameters: Self::Parameters) -> Result<(), Error> {
        Ok(())
    }
    fn website_url(parameters: Self::Parameters) -> String;
    async fn folder_name(
        session: &mut Session,
        parameters: Self::Parameters,
    ) -> Result<String, Error>;
}
