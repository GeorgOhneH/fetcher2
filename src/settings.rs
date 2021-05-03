use crate::template::DownloadArgs;
use config::Config;
use config_derive::Config;
use serde::Serialize;
use std::path::PathBuf;

#[derive(Config, Clone, Serialize, Debug)]
pub struct Settings {
    #[config(ty = "struct")]
    pub downs: DownloadSettings,
}

#[derive(Config, Clone, Serialize, Debug)]
pub struct DownloadSettings {
    pub username: String,
    pub password: String,

    pub save_path: PathBuf,

    #[config(ty = "struct")]
    pub download_args: DownloadArgs,
}
