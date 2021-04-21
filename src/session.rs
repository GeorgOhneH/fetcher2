use reqwest::Client;
use crate::site_modules::Module;
use crate::site_modules::ModuleExt;
use crate::errors::Error;
use std::collections::HashMap;
use tokio::sync::Mutex;
use std::sync::Arc;
use std::borrow::BorrowMut;


#[derive(Clone)]
pub struct Session {
    client: Client,
    login_mutex: Arc<LoginLocks>
}

#[derive(Default)]
pub struct LoginLocks {
    minimal: Mutex<Option<Result<(), Error>>>
}

impl Session {
    pub fn new() -> Self {
        Self {
            client: Client::new(),
            login_mutex: Arc::new(LoginLocks::default()),
        }
    }

    pub async fn login(&self, module: &Module) -> Result<(), Error> {
        match module {
            Module::Minimal(minimal) => {
                let mut lock = self.login_mutex.minimal.lock().await;
                match &*lock {
                    Some(r) => r.clone(),
                    None => {
                        let r = minimal.login(&self).await;
                        let r_clone = r.clone();
                        *lock = Some(r);
                        r_clone
                    }
                }
            }
        }
    }
}
