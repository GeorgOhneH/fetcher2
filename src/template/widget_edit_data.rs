use std::cmp::max;
use std::collections::HashSet;
use std::convert::TryFrom;
use std::fmt::{Debug, Display, Formatter};
use std::path::{Path, PathBuf};
use std::sync::Arc;

use druid::{ExtEventSink, Menu, MenuItem, Rect, Selector, SingleUse, theme, WidgetExt, WidgetId};
use druid::{
    BoxConstraints, Data, Env, Event, EventCtx, LayoutCtx, Lens, LifeCycle, LifeCycleCtx, PaintCtx,
    Point, UpdateCtx, Widget, WidgetPod,
};
use druid::im::Vector;
use druid::kurbo::{BezPath, Size};
use druid::piet::{LineCap, LineJoin, RenderContext, StrokeStyle};
use druid::widget::{Controller, Label};
use druid_widget_nursery::{selectors, Wedge};

use crate::Result;
use crate::template::communication::NODE_EVENT;
use crate::template::nodes::node::NodeEvent;
use crate::template::nodes::node_data::NodeData;
use crate::template::nodes::root_data::RootNodeData;
use crate::template::nodes::root_edit_data::RootNodeEditData;
use crate::template::Template;
use crate::widgets::tree::{DataNodeIndex, NodeIndex, Tree};

#[derive(Debug, Clone, Data, Lens)]
pub struct TemplateEditData {
    pub root: RootNodeEditData,
    #[data(eq)]
    pub save_path: Option<PathBuf>,
}

impl TemplateEditData {
    pub fn new() -> Self {
        Self {
            root: RootNodeEditData::new(),
            save_path: None,
        }
    }
}
