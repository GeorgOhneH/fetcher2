use crate::errors::TemplateError;
use crate::session::Session;
use crate::site_modules::Module;
use crate::task::Task;
use async_recursion::async_recursion;
use async_trait::async_trait;
use config::{Config, ConfigEnum};
use config_derive::Config;
use enum_dispatch::enum_dispatch;
use lazy_static::lazy_static;
use regex::Regex;
use sha1::{Digest, Sha1};
use std::collections::HashMap;
use std::path::Path;
use std::path::PathBuf;
use tokio::fs;

use futures::future::try_join_all;

use crate::settings::DownloadSettings;
use async_std::channel::{self, Receiver, Sender};
use futures::prelude::*;
use serde::Serialize;
use std::sync::Arc;
use std::sync::{Mutex, RwLock};
use tokio::try_join;

use dashmap::mapref::entry::Entry;
use futures::stream::{FuturesUnordered, TryStreamExt};
use reqwest::header::HeaderMap;
use std::ffi::{OsStr, OsString};
use tokio::io::AsyncWriteExt;
use url::Url;

pub async fn extension_from_url(
    session: &Session,
    url: &Url,
) -> Result<Option<OsString>, TemplateError> {
    let response = session.get(url.clone()).send().await?;
    let headers = response.headers();

    if let Some(file_name) = filename_from_headers(headers) {
        Ok(PathBuf::from(file_name)
            .extension()
            .map(|os_str| os_str.to_os_string()))
    } else {
        let extension = headers
            .get_all("content-type")
            .iter()
            .filter_map(|x| x.to_str().ok())
            .flat_map(|mime_str| mime_guess::get_mime_extensions_str(mime_str).into_iter())
            .flatten()
            .next()
            .map(|x| OsString::from(x));
        Ok(extension)
    }
}

pub fn filename_from_headers(headers: &HeaderMap) -> Option<String> {
    lazy_static! {
        static ref FILENAME_RE: Regex = Regex::new("filename=\"(.+)\"").unwrap();
    }
    headers
        .get_all("content-disposition")
        .iter()
        .filter_map(|x| x.to_str().ok())
        .filter_map(|str| FILENAME_RE.captures(str))
        .map(|capture| capture[1].to_owned())
        .next()
}

pub fn add_to_file_stem(path: &PathBuf, name: &str) -> PathBuf {
    let mut file_name = path.file_stem().unwrap().to_os_string();
    file_name.push(name);

    if let Some(extension) = path.extension() {
        file_name.push(".");
        file_name.push(extension);
    };

    path.with_file_name(file_name)
}
