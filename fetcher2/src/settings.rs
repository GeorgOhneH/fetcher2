use std::path::PathBuf;
use config::traveller::Travel;


use crate::error::{Result, TErrorKind};
use crate::template::DownloadArgs;

#[cfg_attr(feature = "druid", derive(druid::Data))]
#[derive(Travel, Debug, Clone)]
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
