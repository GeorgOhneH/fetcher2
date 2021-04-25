use async_std::path::PathBuf;
use url::Url;

#[derive(Debug)]
pub struct Task {
    pub path: PathBuf,
    pub url: Url,
    pub checksum: Option<String>,
}

impl Task {
    fn new(path: PathBuf, url: Url) -> Self {
        Self {
            path,
            url,
            checksum: None
        }
    }
}

pub struct TaskBuilder {
    inner: Task
}

impl TaskBuilder {
    pub fn new(path: PathBuf, url: Url) -> Self {
        Self {
            inner: Task::new(path, url)
        }
    }

    pub fn checksum(mut self, checksum: String) -> Self {
        self.inner.checksum = Some(checksum);
        self
    }

    pub fn build(self) -> Task {
        self.inner
    }
}