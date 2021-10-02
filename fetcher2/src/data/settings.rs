use std::path::PathBuf;

use config::Config;
use druid::{Data, Lens};

use crate::error::{Result, TErrorKind};
use crate::template::DownloadArgs;

#[derive(Clone, Lens, Debug, Data, Config)]
pub struct OptionSettings {
    #[config(ty = "_<struct>")]
    pub settings: Option<Settings>,
}

#[derive(Config, Debug, Data, Clone)]
pub struct Settings {
    #[config(ty = "struct", name = "Download")]
    pub download: DownloadSettings,
}

#[derive(Config, Debug, Data, Clone)]
pub struct DownloadSettings {
    #[config(name = "Username")]
    pub username: Option<String>,

    #[config(name = "Password")]
    pub password: Option<String>,

    #[data(same_fn = "PartialEq::eq")]
    #[config(name = "Save Path")]
    #[config(must_absolute = false, must_exist = false)]
    pub save_path: PathBuf,

    #[config(ty = "struct")]
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
            .ok_or_else(|| TErrorKind::LoginDataRequired.into())
    }
    pub fn try_password(&self) -> Result<&String> {
        self.password
            .as_ref()
            .ok_or_else(|| TErrorKind::LoginDataRequired.into())
    }
}
