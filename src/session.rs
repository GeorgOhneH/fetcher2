use crate::error::{Result, TError};
use crate::site_modules::LoginLocks;
use crate::site_modules::Module;
use crate::site_modules::ModuleExt;
use reqwest::{Client, ClientBuilder, IntoUrl, Method, Request, RequestBuilder, Response};

use crate::settings::DownloadSettings;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Mutex;

#[derive(Clone)]
pub struct Session {
    client: Client,
    pub login_mutex: Arc<LoginLocks>,
}

impl Session {
    pub fn new() -> Self {
        Self {
            client: ClientBuilder::new().cookie_store(true).build().unwrap(),
            login_mutex: Arc::new(LoginLocks::default()),
        }
    }

    pub fn get<U: IntoUrl>(&self, url: U) -> RequestBuilder {
        self.client.get(url)
    }

    pub fn post<U: IntoUrl>(&self, url: U) -> RequestBuilder {
        self.client.post(url)
    }

    pub fn request<U: IntoUrl>(&self, method: Method, url: U) -> RequestBuilder {
        self.client.request(method, url)
    }

    pub async fn execute(&self, request: Request) -> Result<Response> {
        Ok(self.client.execute(request).await?)
    }
}
