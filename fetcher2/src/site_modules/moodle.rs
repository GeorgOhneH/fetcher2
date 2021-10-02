use std::path::PathBuf;
use std::sync::Arc;

use async_trait::async_trait;
use config::Config;
use druid::Data;
use lazy_static::lazy_static;
use soup::{NodeExt, QueryBuilderExt, Soup};
use tokio::sync::mpsc::Sender;
use url::Url;

use crate::data::settings::DownloadSettings;
use crate::error::TErrorFast;
use crate::session::Session;
use crate::site_modules::aai_login::aai_login;
use crate::site_modules::utils::remove_vz_id;
use crate::site_modules::ModuleExt;
use crate::task::Task;
use crate::Result;

static LOGIN_FORM: [(&str, &str); 1] = [("idp", "https://aai-logon.ethz.ch/idp/shibboleth")];

lazy_static! {
    static ref LOGIN_URL: Url =
        Url::parse("https://moodle-app2.let.ethz.ch/auth/shibboleth/login.php").unwrap();
    static ref COURSE_URL: Url =
        Url::parse("https://moodle-app2.let.ethz.ch/course/view.php").unwrap();
}

#[derive(Config, Debug, Clone, Data, PartialEq)]
pub struct Moodle {
    pub id: String,
}

#[async_trait]
impl ModuleExt for Moodle {
    async fn fetch_urls_impl(
        &self,
        _session: Session,
        _sender: Sender<Task>,
        _dsettings: Arc<DownloadSettings>,
    ) -> Result<()> {
        todo!()
    }

    async fn login_impl(&self, session: &Session, dsettings: &DownloadSettings) -> Result<()> {
        aai_login(session, dsettings, LOGIN_URL.clone(), &LOGIN_FORM).await
    }

    fn website_url_impl(&self) -> String {
        todo!()
    }

    async fn folder_name_impl(
        &self,
        session: &Session,
        _dsettings: &DownloadSettings,
    ) -> Result<PathBuf> {
        let mut url = COURSE_URL.clone();
        url.set_query(Some(&format!("id={}", self.id)));

        let response = session.get(url).send().await?;
        let text = response.text().await?;
        let soup = Soup::new(&text);
        let name = soup
            .tag("div")
            .class("page-header-headings")
            .find()
            .wrong_format()?
            .text();
        Ok(PathBuf::from(remove_vz_id(&name).as_ref()))
    }
}
