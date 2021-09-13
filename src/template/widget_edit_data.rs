use std::convert::TryFrom;
use std::fmt::{Debug, Display, Formatter};
use std::sync::Arc;

use druid::kurbo::{BezPath, Size};
use druid::piet::{LineCap, LineJoin, RenderContext, StrokeStyle};
use druid::widget::{Controller, Label};
use druid::{theme, ExtEventSink, Menu, MenuItem, Rect, Selector, SingleUse, WidgetExt, WidgetId};
use druid::{
    BoxConstraints, Data, Env, Event, EventCtx, LayoutCtx, Lens, LifeCycle, LifeCycleCtx, PaintCtx,
    Point, UpdateCtx, Widget, WidgetPod,
};

use crate::delegate::{Msg, MSG_THREAD};
use crate::template::communication::NODE_EVENT;
use crate::template::nodes::node::NodeEvent;
use crate::template::nodes::node_data::NodeData;
use crate::template::nodes::root_data::RootNodeData;
use crate::template::{Template};
use crate::widgets::tree::{DataNodeIndex, Tree, NodeIndex,};
use crate::{AppData, Result};
use druid::im::Vector;
use druid_widget_nursery::{selectors, Wedge};
use std::cmp::max;
use std::collections::HashSet;
use std::path::{Path, PathBuf};
use crate::ui::TemplateInfoSelect;
use crate::template::nodes::root_edit_data::RootNodeEditData;

#[derive(Debug, Clone, Data, Lens)]
pub struct TemplateEditData {
    pub root: RootNodeEditData,
}

impl TemplateEditData {
    pub fn new() -> Self {
        Self {
            root: RootNodeEditData::new()
        }
    }
}