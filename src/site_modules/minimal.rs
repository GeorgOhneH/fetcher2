use crate::errors::TemplateError;
use crate::session::Session;
use crate::task::{Task, TaskBuilder};
use async_trait::async_trait;

use config_derive::Config;
use reqwest::multipart::Form;
use serde::Serialize;
use std::path::{Path, PathBuf};
use tokio::sync::mpsc::Sender;

use crate::settings::DownloadSettings;
use std::collections::HashMap;
use tokio::time::Duration;

use futures::stream::{self, StreamExt, TryStream, TryStreamExt};
use url::Url;

#[derive(Config, Serialize, Debug)]
pub struct Minimal {
    pub parameters: Option<String>,
}

impl Minimal {
    pub async fn retrieve_urls(
        &self,
        session: Session,
        sender: Sender<Task>,
        base_path: PathBuf,
    ) -> Result<(), TemplateError> {
        println!("Retirevinbg Urls");
        let task = TaskBuilder::new(
            base_path.join("hello.hello"),
            Url::parse("https://www.google.com/").unwrap(),
        )
        .build();
        sender.send(task).await.unwrap();
        let resp = session.get("https://www.google.com/").send().await?;
        println!("1{}", resp.status());
        let resp = session.get("https://www.google.com/").send().await?;
        println!("2{}", resp.status());
        //tokio::time::sleep(Duration::from_secs(3)).await;
        let resp = session.get("https://www.google.com/").send().await?;
        println!("3{}", resp.status());
        let task = TaskBuilder::new(
            base_path.join("hello2.hello"),
            Url::parse("https://www.google.com/").unwrap(),
        )
        .extension(false)
        .build();
        sender.send(task).await.unwrap();
        let task = TaskBuilder::new(
            base_path.join("rsgdrf.pdf"),
            Url::parse("http://www.orimi.com/pdf-test.pdf/").unwrap(),
        )
        .build();
        sender.send(task).await.unwrap();
        let task = TaskBuilder::new(
            base_path.join("rs.gdrf"),
            Url::parse("http://www.orimi.com/pdf-test.pdf/").unwrap(),
        )
        .extension(false)
        .build();
        sender.send(task).await.unwrap();

        for x in 0..10 {
            let task = TaskBuilder::new(
                base_path.join(format!("hello.hello{}", x)),
                Url::parse("https://www.google.com/").unwrap(),
            )
            .build();
            sender.send(task).await.unwrap();
            //tokio::time::sleep(Duration::from_millis(100)).await;
        }

        Ok(())
    }

    pub async fn login(
        &self,
        _session: &Session,
        dsettings: &DownloadSettings,
    ) -> Result<(), TemplateError> {
        println!("LOGIN MINIMAL");
        let url =
            url::Url::parse("https://moodle-app2.let.ethz.ch/auth/shibboleth/login.php").unwrap();

        let form = [("idp", "https://aai-logon.ethz.ch/idp/shibboleth")];
        //crate::site_modules::aai_login::aai_login(_session, dsettings, url, &form).await
        Ok(())
    }

    pub fn website_url(&self) -> String {
        "todo!()".to_owned()
    }

    pub async fn folder_name(&self, _session: &Session) -> Result<&Path, TemplateError> {
        println!("Folder Name");
        Ok(Path::new("efgeuif"))
    }
}
