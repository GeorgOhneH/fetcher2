use crate::error::{Result, TError};
use crate::session::Session;
use crate::task::{Task, TaskBuilder};
use async_trait::async_trait;
use html5ever::rcdom::{Handle, NodeData, RcDom};
use soup::prelude::*;

use config::ConfigEnum;
use config_derive::Config;
use reqwest::multipart::Form;
use serde::Serialize;
use std::path::{Path, PathBuf};
use tokio::sync::mpsc::Sender;

use crate::settings::DownloadSettings;
use lazy_static::lazy_static;
use regex::Regex;
use std::collections::HashMap;
use tokio::time::Duration;

use crate::site_modules::ModuleExt;
use futures::stream::{self, StreamExt, TryStream, TryStreamExt};
use url::Url;

lazy_static! {
    static ref INDEX_URL: Url = Url::parse("https://polybox.ethz.ch/index.php/").unwrap();
}

#[derive(Config, Serialize, Debug)]
pub struct Polybox {
    pub id: String,

    #[config(ty = "enum")]
    pub mode: Mode,
}

#[derive(Config, Serialize, Debug)]
pub enum Mode {
    Shared(Option<String>),
    Private,
}

impl Polybox {
    pub async fn html_login(&self, session: &Session, password: &str) -> Result<()> {
        lazy_static! {
            static ref TOKEN_RE: Regex =
                Regex::new("<input .*name=\"requesttoken\" .*value=\"(.*)\".*>").unwrap();
        }
        let url = INDEX_URL
            .join("s/")
            .unwrap()
            .join(&format!("{}/", self.id))?
            .join("authenticate")?;

        println!("{}", url);

        let resp = session.get(url.clone()).send().await?;

        let text = resp.text().await?;
        let token = &TOKEN_RE.captures(&text)?[1];

        let data = [("requesttoken", token), ("password", password)];

        session.post(url).form(&data).send().await?;

        Ok(())
    }
}

#[async_trait]
impl ModuleExt for Polybox {
    async fn retrieve_urls(
        &self,
        session: Session,
        sender: Sender<Task>,
        base_path: PathBuf,
    ) -> Result<()> {
        println!("Retirevinbg Urls");
        //tokio::time::sleep(Duration::from_secs(3)).await;
        let task = TaskBuilder::new(
            base_path.join("hello.hello"),
            Url::parse("https://www.google.com/").unwrap(),
        )
        .build();
        sender.send(task).await?;
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
        sender.send(task).await?;
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
        sender.send(task).await?;

        for x in 0..10 {
            let task = TaskBuilder::new(
                base_path.join(format!("hello.hello{}", x)),
                Url::parse("https://www.google.com/").unwrap(),
            )
            .build();
            sender.send(task).await?;
            //tokio::time::sleep(Duration::from_millis(100)).await;
        }

        Ok(())
    }

    fn website_url(&self) -> String {
        "todo!()".to_owned()
    }

    async fn folder_name(
        &self,
        session: &Session,
        dsettings: &DownloadSettings,
    ) -> Result<PathBuf> {
        match &self.mode {
            Mode::Private => {
                let url = INDEX_URL.join("f/").unwrap().join(&self.id)?;
                let response = session
                    .get(url)
                    .basic_auth(&dsettings.username, Some(&dsettings.password))
                    .send()
                    .await?;

                let (_, dir) = response.url().query_pairs().find(|(key, _)| key == "dir")?;
                let folder_name = dir.split("/").last()?;
                return Ok(PathBuf::from(folder_name));
            }
            Mode::Shared(password) => {
                if let Some(password) = password {
                    self.html_login(session, password).await?;
                }

                let url = INDEX_URL.join("s/").unwrap().join(&self.id)?;

                let response = session
                    .get(url)
                    .basic_auth(&dsettings.username, Some(&dsettings.password))
                    .send()
                    .await?;

                let text = response.text().await?;
                let soup = Soup::new(&text);
                let data_node = soup
                    .tag("body")
                    .find()?
                    .tag("header")
                    .find()?
                    .tag("div")
                    .find()?;
                let name = data_node.get("data-name")?;
                Ok(PathBuf::from(name))
            }
        }
    }
}
