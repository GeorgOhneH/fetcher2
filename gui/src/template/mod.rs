use std::collections::{HashMap, HashSet};
use std::fmt::{Debug, Formatter};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex, RwLock};

use config::{Config, ConfigEnum};
use druid::widget::prelude::*;
use druid::widget::Label;
use druid::{Data, ExtEventSink, Lens, WidgetExt, WidgetId};
use tokio::fs;
use tokio::io::AsyncWriteExt;

use crate::template::communication::{Communication, RawCommunication};
use crate::template::nodes::root_data::RootNodeData;
use crate::template::nodes::root_edit_data::RootNodeEditData;
use crate::template::widget_data::TemplateData;
use crate::template::widget_edit_data::TemplateEditData;

pub mod communication;
pub mod node_type;
pub mod nodes;
pub mod widget_data;
pub mod widget_edit_data;

