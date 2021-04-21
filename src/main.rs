#![allow(dead_code)]

use clap::{AppSettings, Clap};
use config::{CStruct, Config, ConfigEnum, InactiveBehavior};
use config_derive::Config;
use serde::{Deserialize, Serialize};
use serde_yaml::Value;
use async_std::channel;
mod errors;
mod task;
mod session;
mod site_modules;
mod template;

use crate::template::Template;


#[tokio::main]
async fn main() {
    let template = Template::new();
    let session = crate::session::Session::new();
    let (sender, receiver) = channel::bounded(10);
    template.run_root(session, sender).await.unwrap()
}
