use std::path::PathBuf;

use config::Config;
use druid::Data;
use druid::im::Vector;
use serde::Serialize;

use crate::error::{Result, TErrorKind};
use crate::template::DownloadArgs;

#[derive(Config, Serialize, Debug, Data, Clone)]
pub struct Test {
    #[config(default = false)]
    #[config(name = "Force Download")]
    pub force: bool,
}

#[derive(Config, Serialize, Debug, Data, Clone)]
pub struct Settings {
    #[config(ty = "Struct", name = "Download")]
    pub downs: DownloadSettings,
}

#[derive(Config, Serialize, Debug, Data, Clone)]
pub struct DownloadSettings {
    #[config(name = "Username")]
    pub username: Option<String>,

    #[config(name = "Password")]
    pub password: Option<String>,

    #[data(same_fn = "PartialEq::eq")]
    #[config(name = "Save Path")]
    pub save_path: PathBuf,

    #[config(ty = "Struct")]
    #[config(name = "Standard Module Setting")]
    pub download_args: DownloadArgs,

    #[config(default = false)]
    #[config(name = "Force Download")]
    pub force: bool,
}

impl DownloadSettings {
    pub fn try_username(&self) -> Result<&String> {
        self.username
            .as_ref()
            .ok_or(TErrorKind::LoginDataRequired.into())
    }
    pub fn try_password(&self) -> Result<&String> {
        self.password
            .as_ref()
            .ok_or(TErrorKind::LoginDataRequired.into())
    }
}
