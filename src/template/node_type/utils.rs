use std::collections::HashMap;
use std::ffi::{OsStr, OsString};
use std::path::Path;
use std::path::PathBuf;
use std::sync::{Mutex, RwLock};
use std::sync::Arc;

use async_recursion::async_recursion;
use async_trait::async_trait;
use config::{Config, ConfigEnum};
use dashmap::mapref::entry::Entry;
use enum_dispatch::enum_dispatch;
use futures::future::try_join_all;
use futures::prelude::*;
use futures::stream::{FuturesUnordered, TryStreamExt};
use lazy_static::lazy_static;
use regex::Regex;
use reqwest::header::HeaderMap;
use serde::Serialize;
use sha1::{Digest, Sha1};
use tokio::fs;
use tokio::io::AsyncWriteExt;
use tokio::try_join;
use url::Url;

use crate::error::{Result, TError};
use crate::session::Session;
use crate::data::settings::DownloadSettings;
use crate::site_modules::Module;
use crate::task::Task;

pub async fn extension_from_url(session: &Session, url: &Url) -> Result<Option<OsString>> {
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
