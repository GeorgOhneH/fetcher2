use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc::Sender;
use url::Url;

use config::traveller::Travel;

use crate::error::Result;
use crate::session::Session;
use crate::settings::DownloadSettings;
use crate::site_modules::module::ModuleExt;
use crate::task::{Task, TaskBuilder};

#[cfg_attr(feature = "druid", derive(druid::Data, druid::Lens))]
#[derive(Travel, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Minimal {
    pub parameters: Option<String>,
    pub parameters2: Option<String>,
}

#[async_trait]
impl ModuleExt for Minimal {
    async fn fetch_urls_impl(
        &self,
        session: Session,
        sender: Sender<Task>,
        _dsettings: Arc<DownloadSettings>,
    ) -> Result<()> {
        println!("Retirevinbg Urls");
        //tokio::time::sleep(Duration::from_secs(3)).await;
        let task = TaskBuilder::new(
            PathBuf::from("hello.hello"),
            Url::parse("https://www.google.com/").unwrap(),
        )
        .build();
        sender.send(task).await.unwrap();
        let resp = session.get("https://www.google.com/").send().await?;
        println!("1{}", resp.status());
        let resp = session.get("https://www.google.com/").send().await?;
        println!("2{}", resp.status());
        tokio::time::sleep(Duration::from_secs(3)).await;
        let resp = session.get("https://www.google.com/").send().await?;
        println!("3{}", resp.status());
        let task = TaskBuilder::new(
            PathBuf::from("hello2.hello"),
            Url::parse("https://www.google.com/").unwrap(),
        )
        .extension(false)
        .build();
        sender.send(task).await.unwrap();
        let task = TaskBuilder::new(
            PathBuf::from("rsgdrf.pdf"),
            Url::parse("http://www.orimi.com/pdf-test.pdf/").unwrap(),
        )
        .build();
        sender.send(task).await.unwrap();
        let task = TaskBuilder::new(
            PathBuf::from("rs.gdrf"),
            Url::parse("http://www.orimi.com/pdf-test.pdf/").unwrap(),
        )
        .extension(false)
        .build();
        sender.send(task).await.unwrap();

        for x in 0..10 {
            let task = TaskBuilder::new(
                PathBuf::from(format!("hello.hello{}", x)),
                Url::parse("https://www.google.com/").unwrap(),
            )
            .build();
            sender.send(task).await.unwrap();
            //tokio::time::sleep(Duration::from_millis(100)).await;
        }

        Ok(())
    }

    async fn login_impl(&self, session: &Session, dsettings: &DownloadSettings) -> Result<()> {
        println!("LOGIN MINIMAL");
        let url =
            url::Url::parse("https://moodle-app2.let.ethz.ch/auth/shibboleth/login.php").unwrap();

        tokio::time::sleep(Duration::from_secs(3)).await;
        let form = [("idp", "https://aai-logon.ethz.ch/idp/shibboleth")];
        crate::site_modules::aai_login::aai_login(session, dsettings, url, &form).await?;
        Ok(())
    }

    fn website_url_impl(&self) -> String {
        "todo!()".to_owned()
    }

    async fn folder_name_impl(
        &self,
        _session: &Session,
        _dsettings: &DownloadSettings,
    ) -> Result<PathBuf> {
        println!("Folder Name");
        tokio::time::sleep(Duration::from_secs(5)).await;
        Ok(PathBuf::from("efgeuif"))
    }
}
