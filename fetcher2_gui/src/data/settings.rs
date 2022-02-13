use config::traveller::Travel;
use druid::{Data, Lens};
use fetcher2::settings::DownloadSettings;
use serde::{Deserialize, Serialize};

#[derive(Clone, Lens, Debug, Data, Serialize, Deserialize, Default)]
pub struct OptionSettings {
    pub settings: Option<Settings>,
}

#[derive(Travel, Debug, Data, Clone, Serialize, Deserialize)]
pub struct Settings {
    #[travel(name = "Download")]
    pub download: DownloadSettings,
}
