use config::Config;
use druid::{Data, Lens};
use fetcher2::settings::DownloadSettings;

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
