use std::convert::TryFrom;
use std::fmt::{Debug, Display, Formatter};
use std::sync::Arc;

use druid::kurbo::{BezPath, Size};
use druid::piet::{LineCap, LineJoin, RenderContext, StrokeStyle};
use druid::widget::Label;
use druid::{theme, ExtEventSink, Rect, Selector, SingleUse, WidgetId, WidgetExt};
use druid::{
    BoxConstraints, Data, Env, Event, EventCtx, LayoutCtx, LifeCycle, LifeCycleCtx, PaintCtx,
    Point, UpdateCtx, Widget, WidgetPod, Lens,
};

use crate::template::nodes::root_data::{RootNodeData};
use crate::template::Template;
use crate::widgets::{Split, SplitOrBox};
use crate::Result;
use druid::im::Vector;
use druid_widget_nursery::{selectors, Wedge};
use std::cmp::max;
use std::path::{Path, PathBuf};
use crate::widgets::tree::Tree;
use crate::template::nodes::node_data::NodeData;

#[derive(Debug, Clone, Data, Lens)]
pub struct TemplateData {
    pub root: RootNodeData,
    pub selected: Option<Vector<usize>>,
}

impl TemplateData {
    pub fn build_widget() -> impl Widget<Self> {
        Tree::new(
            [
                Label::new("Hello"),
                Label::new("Hello2"),
                Label::new("Hello3"),
            ],
            [
                Arc::new(|| Label::dynamic(|data: &NodeData, _env| data.name())
                    .controller(TemplateUpdate)
                    .boxed()),
                Arc::new(|| Label::dynamic(|data: &NodeData, _env| {
                    let (add, repl) = data.added_replaced();
                    format!("{}|{}", add, repl)
                }).align_right()
                    .boxed()),
                Arc::new(|| Label::dynamic(|data: &NodeData, _| data.state_string()).boxed()),
            ],
            NodeData::expanded,
        ).lens(TemplateData::root)
    }
}
