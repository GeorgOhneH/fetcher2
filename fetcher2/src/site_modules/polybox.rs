use std::path::PathBuf;
use std::str::FromStr;
use std::sync::Arc;

use async_trait::async_trait;
use druid::Data;
use lazy_static::lazy_static;
use quick_xml::events::Event;
use quick_xml::Reader;
use regex::Regex;
use reqwest::header::{HeaderMap, HeaderValue};
use reqwest::Method;
use soup::prelude::*;
use tokio::sync::mpsc::Sender;
use url::Url;

use config::Config;
use config::ConfigEnum;

use crate::error::{Result, TErrorFast, TErrorKind};
use crate::session::Session;
use crate::settings::DownloadSettings;
use crate::site_modules::utils::save_path;
use crate::site_modules::ModuleExt;
use crate::task::{Task, TaskBuilder};

static PROPFIND_DATA: &str = r#"<?xml version="1.0"?>
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
            HeaderValue::from_static("application/xml; charset=utf-8"),
        );
        m.insert("Depth", HeaderValue::from_str("infinity").unwrap());
        m.insert(
            "User-Agent",
            HeaderValue::from_static(
                "Mozilla/5.0 (Windows NT 10.0; Win64; x64; rv:74.0) Gecko/20100101 Firefox/74.0",
            ),
        );
        m
    };
}

#[derive(Config, Debug, Clone, Data, PartialEq)]
pub struct Polybox {
    pub id: String,

    #[config(ty = "enum")]
    pub mode: Mode,
}

#[derive(ConfigEnum, Debug, Clone, Data, PartialEq)]
pub enum Mode {
    Shared(Option<String>),
    Private,
}

impl Polybox {
    pub async fn html_login(&self, session: &Session, password: &str) -> Result<()> {
        lazy_static! {
            static ref TOKEN_RE: Regex = Regex::new("<head data-requesttoken=\"(.*)\">").unwrap();
        }
        let url = INDEX_URL
            .join("s/")
            .unwrap()
            .join(&format!("{}/", self.id))?
            .join("authenticate")?;

        let resp = session.get(url.clone()).send().await?;

        let text = resp.text().await?;
        let token = &TOKEN_RE.captures(&text).wrong_format()?[1];

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
            .basic_auth(dsettings.try_username()?, Some(dsettings.try_password()?))
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
    async fn fetch_urls_impl(
        &self,
        session: Session,
        sender: Sender<Task>,
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
                    .parse(sender, &self.id, password.as_ref(), 3)
                    .await?;
            }
            Mode::Private => {
                let dire_path = self.dire_path(&session, &dsettings).await?;
                let url = USER_WEBDAV_URL
                    .join(&format!("{}/", dsettings.try_username()?))?
                    .join(&dire_path[1..])?;
                let response = session
                    .request(Method::from_str("PROPFIND").unwrap(), url)
                    .basic_auth(dsettings.try_username()?, Some(dsettings.try_password()?))
                    .body(PROPFIND_DATA)
                    .headers(HEADERS.clone())
                    .send()
                    .await?
                    .error_for_status()?;

                let xml = response.text().await?;

                let n_skip = 5 + dire_path
                    .split('/')
                    .map(|x| x.trim())
                    .filter(|x| !x.is_empty())
                    .count();

                XmlReader::new(&xml)
                    .parse(
                        sender,
                        dsettings.try_username()?,
                        Some(dsettings.try_password()?),
                        n_skip,
                    )
                    .await?;
            }
        };

        Ok(())
    }

    fn website_url_impl(&self) -> String {
        "todo!()".to_owned()
    }

    async fn folder_name_impl(
        &self,
        session: &Session,
        dsettings: &DownloadSettings,
    ) -> Result<PathBuf> {
        match &self.mode {
            Mode::Private => {
                let dir_path = self.dire_path(session, dsettings).await?;
                let folder_name = dir_path.split('/').last().wrong_format()?;
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
                    .basic_auth(dsettings.try_username()?, Some(dsettings.try_password()?))
                    .send()
                    .await?;

                let text = response.text().await?;
                let soup = Soup::new(&text);
                let data_node = soup
                    .tag("body")
                    .find()
                    .wrong_format()?
                    .tag("header")
                    .find()
                    .wrong_format()?
                    .tag("div")
                    .find()
                    .wrong_format()?;
                let name = data_node.get("data-name").wrong_format()?;
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
        username: &str,
        password: Option<&String>,
        n_skip: usize,
    ) -> Result<()> {
        loop {
            match self.read_event()? {
                Event::Start(ref e) => {
                    if let b"d:response" = e.name() {
                        let r = self.parse_response()?;
                        if r.status != "HTTP/1.1 200 OK" {
                            continue;
                        }

                        let path = r
                            .href
                            .split('/')
                            .skip(n_skip)
                            .map(|part| save_path(part))
                            .collect::<Result<PathBuf>>()?;

                        let url = BASE_URL.join(&r.href)?;

                        let task = TaskBuilder::new(path, url)
                            .checksum(r.checksum)
                            .basic_auth(username.to_owned(), password.map(Clone::clone))
                            .build();

                        sender.send(task).await.unwrap();
                    }
                }
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
                Event::End(ref e) => {
                    if let b"d:response" = e.name() {
                        return Ok(resp);
                    }
                }
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
