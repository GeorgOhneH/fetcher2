use crate::error::{Result, TError, TErrorKind};
use crate::session::Session;
use crate::task::{Task, TaskBuilder};
use async_trait::async_trait;
use html5ever::rcdom::{Handle, NodeData, RcDom};
use soup::prelude::*;

use config::ConfigEnum;
use config_derive::Config;
use quick_xml::events::{BytesText, Event};
use quick_xml::Reader;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use tokio::sync::mpsc::Sender;

use crate::settings::DownloadSettings;
use lazy_static::lazy_static;
use regex::Regex;
use std::collections::HashMap;
use tokio::time::Duration;

use crate::site_modules::utils::save_path;
use crate::site_modules::ModuleExt;
use futures::future::ok;
use futures::stream::{self, StreamExt, TryStream, TryStreamExt};
use reqwest::header::{HeaderMap, HeaderValue};
use reqwest::Method;
use std::str::FromStr;
use std::sync::Arc;
use url::Url;

static PROPFIND_DATA: &'static str = r#"<?xml version="1.0"?>
    <a:propfind xmlns:a="DAV:">
        <a:prop xmlns:oc="http://owncloud.org/ns">
            <oc:checksums/>
        </a:prop>
    </a:propfind>"#;

lazy_static! {
    static ref BASE_URL: Url = Url::parse("https://polybox.ethz.ch").unwrap();
    static ref INDEX_URL: Url = Url::parse("https://polybox.ethz.ch/index.php/").unwrap();
    static ref USER_WEBDAV_URL: Url =
        Url::parse("https://polybox.ethz.ch/remote.php/dav/files/").unwrap();
    static ref WEBDAV_PUBLIC_URL: Url =
        Url::parse("https://polybox.ethz.ch/public.php/webdav/").unwrap();
    static ref WEBDAV_REMOTE_URL: Url =
        Url::parse("https://polybox.ethz.ch/remote.php/webdav/").unwrap();
    static ref HEADERS: HeaderMap = {
        let mut m = HeaderMap::with_capacity(3);
        m.insert(
            "Content-Type",
            HeaderValue::from_str("application/xml; charset=utf-8").unwrap(),
        );
        m.insert("Depth", HeaderValue::from_str("infinity").unwrap());
        m.insert(
            "User-Agent",
            HeaderValue::from_str(
                "Mozilla/5.0 (Windows NT 10.0; Win64; x64; rv:74.0) Gecko/20100101 Firefox/74.0",
            )
            .unwrap(),
        );
        m
    };
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

        let resp = session.get(url.clone()).send().await?;

        let text = resp.text().await?;
        let token = &TOKEN_RE.captures(&text)?[1];

        let data = [("requesttoken", token), ("password", password)];

        session
            .post(url)
            .form(&data)
            .send()
            .await?
            .error_for_status()?;

        Ok(())
    }

    pub async fn dire_path(
        &self,
        session: &Session,
        dsettings: &DownloadSettings,
    ) -> Result<String> {
        let url = INDEX_URL.join("f/").unwrap().join(&self.id)?;
        let response = session
            .get(url)
            .basic_auth(&dsettings.username, Some(&dsettings.password))
            .send()
            .await?;

        let (_, dir) = response
            .url()
            .query_pairs()
            .find(|(key, _)| key == "dir")
            .ok_or(TErrorKind::LoginError)?;
        Ok(dir.into_owned())
    }
}

#[async_trait]
impl ModuleExt for Polybox {
    async fn retrieve_urls(
        &self,
        session: Session,
        sender: Sender<Task>,
        base_path: PathBuf,
        dsettings: Arc<DownloadSettings>,
    ) -> Result<()> {
        match &self.mode {
            Mode::Shared(password) => {
                println!("STARTING {:?}", password);
                let response = session
                    .request(
                        Method::from_str("PROPFIND").unwrap(),
                        WEBDAV_PUBLIC_URL.clone(),
                    )
                    .basic_auth(&self.id, password.as_ref())
                    .body(PROPFIND_DATA)
                    .headers(HEADERS.clone())
                    .send()
                    .await?;

                let xml = response.text().await?;
                XmlReader::new(&xml)
                    .parse(sender, base_path, &self.id, password.as_ref(), 3)
                    .await?;
            }
            Mode::Private => {
                let dire_path = self.dire_path(&session, &dsettings).await?;
                let url = USER_WEBDAV_URL
                    .join(&format!("{}/", dsettings.username))?
                    .join(&dire_path[1..])?;
                let response = session
                    .request(Method::from_str("PROPFIND").unwrap(), url)
                    .basic_auth(&dsettings.username, Some(&dsettings.password))
                    .body(PROPFIND_DATA)
                    .headers(HEADERS.clone())
                    .send()
                    .await?
                    .error_for_status()?;

                let xml = response.text().await?;

                let n_skip = 5 + dire_path
                    .split("/")
                    .map(|x| x.trim())
                    .filter(|x| !x.is_empty())
                    .count();

                XmlReader::new(&xml)
                    .parse(
                        sender,
                        base_path,
                        &dsettings.username,
                        Some(&dsettings.password),
                        n_skip,
                    )
                    .await?;
            }
        };

        Ok(())
    }

    fn website_url(&self, dsettings: &DownloadSettings) -> String {
        "todo!()".to_owned()
    }

    async fn folder_name(
        &self,
        session: &Session,
        dsettings: &DownloadSettings,
    ) -> Result<PathBuf> {
        match &self.mode {
            Mode::Private => {
                let dir_path = self.dire_path(session, dsettings).await?;
                let folder_name = dir_path.split("/").last()?;
                return Ok(PathBuf::from(folder_name));
            }
            Mode::Shared(password) => {
                // polybox doesn't work without a new session  ¯\_(ツ)_/¯
                let new_session = Session::new();

                if let Some(password) = password {
                    self.html_login(&new_session, password).await?;
                }

                let url = INDEX_URL.join("s/").unwrap().join(&self.id)?;

                let response = new_session
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

struct XmlReader<'a> {
    reader: Reader<&'a [u8]>,
    buf: Vec<u8>,
}

impl<'a> XmlReader<'a> {
    fn new(xml: &'a str) -> Self {
        let mut reader = Reader::from_str(xml);
        reader.trim_text(true);
        Self {
            reader,
            buf: Vec::new(),
        }
    }

    fn read_event(&mut self) -> Result<Event> {
        match self.reader.read_event(&mut self.buf) {
            Ok(event) => Ok(event),
            Err(err) => Err(TErrorKind::Xml(format!(
                "Error at position {}: {}",
                self.reader.buffer_position(),
                err
            ))
            .into()),
        }
    }

    fn read_text(&mut self) -> Result<String> {
        let byte_text = match self.read_event()? {
            Event::Text(byte_text) => byte_text,
            _ => return Err(TErrorKind::Xml("Expected Event::Text".to_owned()).into()),
        };

        byte_text
            .into_owned()
            .unescape_and_decode(&self.reader)
            .map_err(|err| TErrorKind::Xml(format!("Could not decode: {}", err)).into())
    }

    async fn parse(
        &mut self,
        sender: Sender<Task>,
        base_path: PathBuf,
        username: &String,
        password: Option<&String>,
        n_skip: usize,
    ) -> Result<()> {
        loop {
            match self.read_event()? {
                Event::Start(ref e) => match e.name() {
                    b"d:response" => {
                        let r = self.parse_response()?;
                        if r.status != "HTTP/1.1 200 OK" {
                            continue;
                        }

                        let sub_path = r
                            .href
                            .split("/")
                            .skip(n_skip)
                            .map(|part| save_path(part))
                            .collect::<Result<PathBuf>>()?;

                        let path = base_path.join(sub_path);
                        let url = BASE_URL.join(&r.href)?;

                        let task = TaskBuilder::new(path, url)
                            .checksum(r.checksum)
                            .basic_auth(username.clone(), password.map(Clone::clone))
                            .build();

                        sender.send(task).await?;
                    }
                    _ => {}
                },
                Event::Eof => break,
                _ => (),
            }
        }
        Ok(())
    }

    fn parse_response(&mut self) -> Result<Response> {
        let mut resp = Response::default();
        loop {
            match self.read_event()? {
                Event::Start(ref e) => match e.name() {
                    b"d:href" => resp.href = self.read_text()?,
                    b"d:status" => resp.status = self.read_text()?,
                    b"oc:checksum" => resp.checksum = self.read_text()?,
                    _ => {}
                },
                Event::End(ref e) => match e.name() {
                    b"d:response" => return Ok(resp),
                    _ => {}
                },
                _ => {}
            }
        }
    }
}

#[derive(Default, Debug)]
struct Response {
    status: String,
    checksum: String,
    href: String,
}