use std::path::PathBuf;
use std::sync::Arc;

use async_recursion::async_recursion;
use config::{Config, ConfigEnum};
use druid::{Data, ExtEventSink, Widget, WidgetExt, WidgetId};
use druid::im::Vector;
use futures::future::try_join_all;
use futures::prelude::*;
use serde::Serialize;
use sha1::Digest;

use crate::error::Result;
use crate::session::Session;
use crate::data::settings::DownloadSettings;
use crate::template::node_type::{NodeType, NodeTypeData};

pub mod node;
pub mod node_data;
pub mod node_edit_data;
pub mod root;
pub mod root_data;
pub mod root_edit_data;
