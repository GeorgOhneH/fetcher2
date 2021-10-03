use std::path::PathBuf;
use std::sync::Arc;

use async_recursion::async_recursion;
use config::{Config, ConfigEnum};
use druid::im::Vector;
use druid::{Data, ExtEventSink, Widget, WidgetExt, WidgetId};
use futures::future::try_join_all;
use futures::prelude::*;
use serde::Serialize;
use sha1::Digest;


pub mod node_data;
pub mod node_edit_data;
pub mod root_data;
pub mod root_edit_data;
