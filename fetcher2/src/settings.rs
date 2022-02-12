use config::ctypes::path::{Absolute, StrictPath};
use config::traveller::Travel;

use crate::error::{Result, TErrorKind};
use crate::template::DownloadArgs;

#[cfg_attr(feature = "druid", derive(druid::Data))]
#[derive(Travel, Debug, Clone)]
pub struct DownloadSettings {
    #[travel(name = "Username")]
    pub username: Option<String>,

    #[travel(name = "Password")]
    pub password: Option<String>,

    #[travel(name = "Save Path")]
    pub save_path: StrictPath<Absolute>,

    #[travel(name = "Standard Module Setting")]
    pub download_args: DownloadArgs,

    #[travel(default = false)]
    #[travel(name = "Force Download")]
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
