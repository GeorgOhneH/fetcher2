#![allow(dead_code)]

use async_std::channel;
mod errors;
mod session;
mod settings;
mod site_modules;
mod task;
mod template;

use crate::settings::DownloadSettings;
use crate::template::{Template, DownloadArgs};
use std::path::PathBuf;

#[tokio::main]
async fn main() {
    let template = Template::new();
    let session = crate::session::Session::new();
    let dsettings = DownloadSettings {
        username: "gshwan".to_owned(),
        password: "".to_owned(),
        save_path: PathBuf::new(),
        download_args: DownloadArgs {
            allowed_extensions: vec![],
            forbidden_extensions: vec![]
        },
    };
    template.run_root(session, &dsettings).await.unwrap()
}
