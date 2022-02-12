use druid::Data;
use druid::im::Vector;
use fetcher2::template::node_type::site::{
    DownloadEvent, LoginEvent, MsgKind, RunEvent, SiteEvent, TaskMsg, UrlFetchEvent,
};
use fetcher2::TError;
use std::sync::Arc;

use crate::data::template::nodes::node::CurrentState;

#[derive(Debug, Clone, Data)]
pub struct SiteState {
    pub run: usize,
    pub login: LoginState,
    pub fetch: FetchState,
    pub download: DownloadState,
}

impl Default for SiteState {
    fn default() -> Self {
        Self::new()
    }
}

impl SiteState {
    pub fn new() -> Self {
        Self {
            run: 0,
            login: LoginState::new(),
            fetch: FetchState::new(),
            download: DownloadState::new(),
        }
    }

    pub fn reset(&mut self) {
        self.run = 0;
        self.login.reset();
        self.fetch.reset();
        self.download.reset();
    }

    pub fn update(&mut self, event: SiteEvent, history: &mut Vector<TaskMsg>) {
        match event {
            SiteEvent::Run(run_event) => match run_event {
                RunEvent::Start => self.run += 1,
                RunEvent::Finish => self.run -= 1,
            },
            SiteEvent::Login(login_event) => self.login.update(login_event),
            SiteEvent::UrlFetch(fetch_event) => self.fetch.update(fetch_event),
            SiteEvent::Download(down_event) => self.download.update(down_event, history),
        }
    }

    pub fn run_state(&self) -> CurrentState {
        if self.run == 0 {
            CurrentState::Idle
        } else {
            CurrentState::Active("Cleaning Up".into())
        }
    }
}

#[derive(Debug, Clone, Data)]
pub struct LoginState {
    pub count: usize,
    pub errs: Vector<Arc<TError>>,
}

impl LoginState {
    pub fn new() -> Self {
        Self {
            count: 0,
            errs: Vector::new(),
        }
    }

    pub fn reset(&mut self) {
        self.count = 0;
        self.errs.clear();
    }

    pub fn update(&mut self, event: LoginEvent) {
        match event {
            LoginEvent::Start => self.count += 1,
            LoginEvent::Finish => self.count -= 1,
            LoginEvent::Err(err) => {
                self.errs.push_back(Arc::new(err));
                self.count -= 1
            }
        }
    }

    pub fn current_state(&self) -> CurrentState {
        if self.count != 0 {
            CurrentState::Active("Logging in".into())
        } else if !self.errs.is_empty() {
            CurrentState::Error("Error while logging in".into())
        } else {
            CurrentState::Idle
        }
    }
}

#[derive(Debug, Clone, Data)]
pub struct FetchState {
    pub count: usize,
    pub errs: Vector<Arc<TError>>,
}

impl FetchState {
    pub fn new() -> Self {
        Self {
            count: 0,
            errs: Vector::new(),
        }
    }

    pub fn reset(&mut self) {
        self.count = 0;
        self.errs.clear();
    }

    pub fn update(&mut self, event: UrlFetchEvent) {
        match event {
            UrlFetchEvent::Start => self.count += 1,
            UrlFetchEvent::Finish => self.count -= 1,
            UrlFetchEvent::Err(err) => {
                self.errs.push_back(Arc::new(err));
                self.count -= 1
            }
        }
    }

    pub fn current_state(&self) -> CurrentState {
        if self.count != 0 {
            CurrentState::Active("Fetching Urls".into())
        } else if !self.errs.is_empty() {
            CurrentState::Error("Error while fetching Urls".into())
        } else {
            CurrentState::Idle
        }
    }
}

#[derive(Debug, Clone, Data)]
pub struct DownloadState {
    pub count: usize,
    pub total: usize,
    pub new_added: usize,
    pub new_replaced: usize,
    pub errs: Vector<Arc<TError>>,
}

impl DownloadState {
    pub fn new() -> Self {
        Self {
            count: 0,
            total: 0,
            new_added: 0,
            new_replaced: 0,
            errs: Vector::new(),
        }
    }

    pub fn reset(&mut self) {
        self.count = 0;
        self.total = 0;
        self.errs.clear();
    }

    pub fn update(&mut self, event: DownloadEvent, history: &mut Vector<TaskMsg>) {
        match event {
            DownloadEvent::Start => {
                self.count += 1;
                self.total += 1
            }
            DownloadEvent::Finish(msg) => {
                match &msg {
                    TaskMsg {
                        kind: MsgKind::AddedFile,
                        ..
                    } => self.new_added += 1,
                    TaskMsg {
                        kind: MsgKind::ReplacedFile(_),
                        ..
                    } => self.new_replaced += 1,
                    _ => {}
                }
                history.push_back(msg);
                self.count -= 1;
            }
            DownloadEvent::Err(err) => {
                self.errs.push_back(Arc::new(err));
                self.count -= 1
            }
        }
        if self.count == 0 {
            self.total = 0;
        }
    }

    pub fn current_state(&self) -> CurrentState {
        if self.count != 0 {
            CurrentState::Active(
                format!("Processing {}/{}", self.total - self.count, self.total).into(),
            )
        } else if !self.errs.is_empty() {
            CurrentState::Error("Error while downloading files".into())
        } else {
            CurrentState::Idle
        }
    }
}
