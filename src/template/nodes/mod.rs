pub mod root;
pub mod root_data;
pub mod node;
pub mod node_data;

use crate::error::Result;
use crate::session::Session;
use async_recursion::async_recursion;
use config::{Config, ConfigEnum};
use config_derive::Config;
use druid::im::Vector;
use druid::{Data, ExtEventSink, Widget, WidgetExt, WidgetId};
use sha1::Digest;
use std::path::PathBuf;

use futures::future::try_join_all;

use crate::settings::DownloadSettings;
use futures::prelude::*;
use serde::Serialize;
use std::sync::Arc;

use crate::template::node_type::{NodeType, NodeTypeData};


