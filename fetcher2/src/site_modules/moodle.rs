use std::path::PathBuf;
use std::sync::Arc;

use async_trait::async_trait;
use lazy_static::lazy_static;
use regex::Regex;
use serde::{Deserialize, Serialize};
use soup::{NodeExt, QueryBuilderExt, Soup};
use tokio::sync::mpsc::Sender;
use url::Url;
use config::traveller::Travel;

use crate::error::TErrorFast;
use crate::session::Session;
use crate::settings::DownloadSettings;
use crate::site_modules::aai_login::aai_login;
use crate::site_modules::utils::remove_vz_id;
use crate::task::Task;
use crate::Result;
use crate::site_modules::module::ModuleExt;

static LOGIN_FORM: [(&str, &str); 1] = [("idp", "https://aai-logon.ethz.ch/idp/shibboleth")];

lazy_static! {
    static ref LOGIN_URL: Url =
        Url::parse("https://moodle-app2.let.ethz.ch/auth/shibboleth/login.php").unwrap();
    static ref COURSE_URL: Url =
        Url::parse("https://moodle-app2.let.ethz.ch/course/view.php").unwrap();

    static ref SECTION_RE: Regex =
        Regex::new("section-[0-9]+").unwrap();
}

#[cfg_attr(feature = "druid", derive(druid::Data))]
#[derive(Travel, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Moodle {
    pub id: String,
}

impl Moodle {
    async fn parse_section(&self) {

    }
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
        // url.set_query(Some(&format!("id={}", self.id)));
        //
        // let response = session.get(url).send().await?;
        // let text = response.text().await?;
        // let soup = Soup::new(&text);
        //
        // soup.tag("li").attr("id", SECTION_RE).find_all()


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
