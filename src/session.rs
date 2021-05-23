use crate::error::{Result, TError, TErrorKind};
use crate::site_modules::LoginLocks;
use crate::site_modules::Module;
use crate::site_modules::ModuleExt;
use reqwest::{Body, Client, ClientBuilder, IntoUrl, Method, Request, RequestBuilder, Response};

use crate::settings::DownloadSettings;
use reqwest::header::{HeaderMap, HeaderName, HeaderValue};
use serde::Serialize;
use std::convert::TryFrom;
use std::fmt::Display;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Mutex;

#[derive(Clone)]
pub struct Session {
    client: Client,
    pub login_mutex: Arc<LoginLocks>,
}

pub struct SRequestBuilder {
    inner: RequestBuilder,
    session: Session,
}

impl Session {
    pub fn new() -> Self {
        Self {
            client: ClientBuilder::new()
                .cookie_store(true)
                .connect_timeout(Duration::from_secs(10))
                .build()
                .unwrap(),
            login_mutex: Arc::new(LoginLocks::default()),
        }
    }

    pub fn get<U: IntoUrl>(&self, url: U) -> SRequestBuilder {
        self.request(Method::GET, url)
    }

    pub fn post<U: IntoUrl>(&self, url: U) -> SRequestBuilder {
        self.request(Method::POST, url)
    }

    pub fn request<U: IntoUrl>(&self, method: Method, url: U) -> SRequestBuilder {
        SRequestBuilder {
            inner: self.client.request(method, url),
            session: self.clone(),
        }
    }

    pub async fn execute(&self, request: Request) -> Result<Response> {
        let cloneable = request
            .body()
            .map(|b| b.as_bytes().is_some())
            .unwrap_or(true);
        if cloneable {
            self.retry_execute(request).await
        } else {
            self._execute(request).await
        }
    }

    // request must be cloneable
    async fn retry_execute(&self, request: Request) -> Result<Response> {
        for i in 0..4 {
            match self._execute(request.try_clone().unwrap()).await {
                Ok(response) => return Ok(response),
                Err(err) => {
                    println!("{}, {:?}", i, err)
                }
            }
            tokio::time::sleep(Duration::from_secs(3)).await;
        }

        self._execute(request).await
    }

    async fn _execute(&self, request: Request) -> Result<Response> {
        Ok(tokio::time::timeout(Duration::from_secs(30), self.client.execute(request)).await??)
    }
}

impl SRequestBuilder {
    pub fn header<K, V>(mut self, key: K, value: V) -> Self
    where
        HeaderName: TryFrom<K>,
        <HeaderName as TryFrom<K>>::Error: Into<http::Error>,
        HeaderValue: TryFrom<V>,
        <HeaderValue as TryFrom<V>>::Error: Into<http::Error>,
    {
        self.inner = self.inner.header(key, value);
        self
    }

    pub fn headers(mut self, headers: HeaderMap) -> Self {
        self.inner = self.inner.headers(headers);
        self
    }

    pub fn basic_auth<U, P>(mut self, username: U, password: Option<P>) -> Self
    where
        U: Display,
        P: Display,
    {
        self.inner = self.inner.basic_auth(username, password);
        self
    }

    pub fn bearer_auth<T>(mut self, token: T) -> Self
    where
        T: Display,
    {
        self.inner = self.inner.bearer_auth(token);
        self
    }

    pub fn body<T: Into<Body>>(mut self, body: T) -> Self {
        self.inner = self.inner.body(body);
        self
    }

    pub fn timeout(mut self, timeout: Duration) -> Self {
        self.inner = self.inner.timeout(timeout);
        self
    }

    pub fn query<T: Serialize + ?Sized>(mut self, query: &T) -> Self {
        self.inner = self.inner.query(query);
        self
    }

    pub fn form<T: Serialize + ?Sized>(mut self, form: &T) -> Self {
        self.inner = self.inner.form(form);
        self
    }

    pub fn json<T: Serialize + ?Sized>(mut self, json: &T) -> Self {
        self.inner = self.inner.json(json);
        self
    }

    pub fn build(self) -> Result<Request> {
        Ok(self.inner.build()?)
    }

    pub async fn send(self) -> Result<Response> {
        let request = self.inner.build()?;
        self.session.execute(request).await
    }
}
