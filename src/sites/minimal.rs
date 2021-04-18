use crate::errors::Error;
use crate::queue::Queue;
use crate::session::Session;
use crate::sites::Site;
use async_trait::async_trait;
use config::Config;
use config_derive::Config;
use serde::Serialize;
use std::path::PathBuf;

pub struct Minimal {}

#[derive(Config, Clone, Serialize)]
pub struct MinimalParameters {}

#[async_trait]
impl Site for Minimal {
    type Parameters = MinimalParameters;
    async fn retrieve_urls(
        session: &mut Session,
        queue: &mut Queue,
        base_path: PathBuf,
        parameters: MinimalParameters,
    ) -> Result<(), Error> {
        Ok(())
    }

    fn website_url(parameters: MinimalParameters) -> String {
        "todo!()".to_owned()
    }

    async fn folder_name(
        session: &mut Session,
        parameters: MinimalParameters,
    ) -> Result<String, Error> {
        Ok("todo".to_owned())
    }
}
