use std::path::PathBuf;

use reqwest::header::HeaderMap;
use url::Url;

#[derive(Debug)]
pub struct Task {
    pub path: PathBuf,
    pub url: Url,
    pub headers: Option<HeaderMap>,
    pub basic_auth: Option<(String, Option<String>)>,
    pub bearer_auth: Option<String>,
    pub checksum: Option<String>,
    pub has_extension: bool,
}

impl Task {
    fn new(path: PathBuf, url: Url) -> Self {
        Self {
            path,
            url,
            headers: None,
            basic_auth: None,
            bearer_auth: None,
            checksum: None,
            has_extension: true,
        }
    }
}

pub struct TaskBuilder {
    inner: Task,
}

impl TaskBuilder {
    pub fn new(path: PathBuf, url: Url) -> Self {
        Self {
            inner: Task::new(path, url),
        }
    }

    pub fn checksum(mut self, checksum: String) -> Self {
        self.inner.checksum = Some(checksum);
        self
    }

    pub fn extension(mut self, has_extension: bool) -> Self {
        self.inner.has_extension = has_extension;
        self
    }

    pub fn headers(mut self, headers: HeaderMap) -> Self {
        self.inner.headers = Some(headers);
        self
    }

    pub fn basic_auth(mut self, username: String, password: Option<String>) -> Self {
        self.inner.basic_auth = Some((username, password));
        self
    }

    pub fn bearer_auth(mut self, token: String) -> Self {
        self.inner.bearer_auth = Some(token);
        self
    }

    pub fn build(self) -> Task {
        self.inner
    }
}
