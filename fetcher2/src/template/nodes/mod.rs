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

use crate::error::Result;
use crate::session::Session;
use crate::template::node_type::{NodeType};

pub mod node;
pub mod root;
