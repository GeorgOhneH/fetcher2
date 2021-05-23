#![allow(dead_code)]
#![feature(try_trait)]
#![feature(backtrace)]

mod error;
mod session;
mod settings;
mod site_modules;
mod task;
mod template;

pub use error::{Result, TError};

use crate::settings::DownloadSettings;
use crate::template::{differ, DownloadArgs, Extensions, Mode, Template};
use config::Config;
use config_derive::Config;
use futures::StreamExt;
use log::{debug, error, info, log_enabled, Level};
use serde::Serialize;
use std::collections::HashSet;
use std::error::Error;
use std::io;
use std::ops::{Deref, DerefMut};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex, RwLock};
use std::time::Instant;
use tokio::sync::mpsc::{Receiver, Sender};

fn main() {
    let mut base_config = fern::Dispatch::new();

    base_config = base_config.level(log::LevelFilter::Trace);

    let stdout_config = fern::Dispatch::new()
        .filter(|metadata| {
            !(metadata.target().starts_with("html5ever")
                || !metadata.target().starts_with("mio")
                || !metadata.target().starts_with("reqwest::connect"))
        })
        .format(|out, message, record| {
            out.finish(format_args!(
                "[{}][{}][{}] {}",
                chrono::Local::now().format("%H:%M"),
                record.target(),
                record.level(),
                message
            ))
        })
        .chain(io::stdout());

    base_config.chain(stdout_config).apply().unwrap();

    info!("the answer was: ef");
    let mut template = Template::new();
    tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .unwrap()
        .block_on(async {
            let session = crate::session::Session::new();
            let dsettings = DownloadSettings {
                username: std::env::var("USERNAME").unwrap(),
                password: std::env::var("PASSWORD").unwrap(),
                save_path: PathBuf::from("C:\\programming\\rust\\fetcher2\\test"),
                download_args: DownloadArgs {
                    extensions: Extensions {
                        inner: HashSet::new(),
                        mode: Mode::Forbidden,
                    },
                    keep_old_files: true,
                },
                force: false,
            };
            let start = Instant::now();
            match template.run_root(session, dsettings).await {
                Ok(()) => {}
                Err(err) => {
                    print!("{:?}", err);
                    println!("{}", err.backtrace().unwrap());
                    return;
                }
            };
            println!("{:#?}", start.elapsed());
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
