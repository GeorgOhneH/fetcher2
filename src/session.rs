use reqwest::{Client, RequestBuilder, IntoUrl, Method, Request, Response, ClientBuilder};
use crate::site_modules::Module;
use crate::errors::TemplateError;

use tokio::sync::Mutex;
use std::sync::Arc;
use crate::settings::DownloadSettings;


#[derive(Clone)]
pub struct Session {
    client: Client,
    pub login_mutex: Arc<LoginLocks>
}

#[derive(Default)]
pub struct LoginLocks {
    minimal: Mutex<LoginState>,
    pub aai_login: Mutex<()>,
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

impl Session {
    pub fn new() -> Self {
        Self {
            client: ClientBuilder::new()
                .cookie_store(true)
                .build()
                .unwrap(),
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

    pub async fn execute(
        &self,
        request: Request
    ) -> Result<Response, reqwest::Error> {
        self.client.execute(request).await
    }

    pub async fn login(&self, module: &Module, dsettings: &DownloadSettings) -> Result<(), TemplateError> {
        use LoginState::*;
        match module {
            Module::Minimal(minimal) => {
                let mut lock = self.login_mutex.minimal.lock().await;
                match &*lock {
                    Success => Ok(()),
                    Failure => Err(TemplateError::PreviousLoginError),
                    Uninitiated => {
                        let r = minimal.login(&self, dsettings).await;
                        *lock = if r.is_ok() {Success} else {Failure};
                        r
                    },
                }
            }
        }
    }
}
