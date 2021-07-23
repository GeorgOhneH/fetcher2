use crate::template::DownloadArgs;
use config::Config;
use config_derive::Config;
use serde::Serialize;
use std::path::PathBuf;

#[derive(Config, Serialize, Debug)]
pub struct Settings {
    #[config(ty = "Struct")]
    pub downs: DownloadSettings,
}

#[derive(Config, Serialize, Debug)]
pub struct DownloadSettings {
    pub username: String,
    pub password: String,

    pub save_path: PathBuf,

    #[config(ty = "Struct")]
    pub download_args: DownloadArgs,

    #[config(default = false)]
    pub force: bool,
}
