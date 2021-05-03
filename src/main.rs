#![allow(dead_code)]

use async_std::channel;
mod errors;
mod session;
mod settings;
mod site_modules;
mod task;
mod template;

use crate::settings::DownloadSettings;
use crate::template::{DownloadArgs, Template};
use config::Config;
use config_derive::Config;
use futures::StreamExt;
use serde::Serialize;
use std::ops::{Deref, DerefMut};
use std::path::{Path, PathBuf};
use std::sync::{Mutex, RwLock, Arc};

fn main() {
    let mut template = Template::new();
    tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .unwrap()
        .block_on(async {
            let session = crate::session::Session::new();
            let dsettings = DownloadSettings {
                username: "gshwan".to_owned(),
                password: "".to_owned(),
                save_path: PathBuf::from("C:\\programming\\rust\\fetcher2\\test"),
                download_args: DownloadArgs {
                    allowed_extensions: vec![],
                    forbidden_extensions: vec![],
                },
            };
            template.run_root(session, dsettings).await.unwrap();
            let save_path = PathBuf::from("C:\\programming\\rust\\fetcher2\\test.yml");
            template.save(&save_path).await.unwrap();
            template.load(&save_path).await.unwrap();

        });
}

// #[derive(Config, Serialize)]
// pub enum SiteStorage {
//     #[config(inner_ty = "struct")]
//     Hello2(Arc<Hello>),
//     Single
// }
//
//
// #[derive(Config, Serialize, Debug)]
// pub struct Hello {
//
// }
//
//
//
//
// fn main() {}

