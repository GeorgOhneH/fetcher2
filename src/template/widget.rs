use std::convert::TryFrom;
use std::fmt::{Debug, Display, Formatter};
use std::sync::Arc;

use druid::kurbo::{BezPath, Size};
use druid::piet::{LineCap, LineJoin, RenderContext, StrokeStyle};
use druid::widget::{Controller, Label};
use druid::{theme, ExtEventSink, Rect, Selector, SingleUse, WidgetExt, WidgetId};
use druid::{
    BoxConstraints, Data, Env, Event, EventCtx, LayoutCtx, Lens, LifeCycle, LifeCycleCtx, PaintCtx,
    Point, UpdateCtx, Widget, WidgetPod,
};

use crate::template::communication::NODE_EVENT;
use crate::template::nodes::node::NodeEvent;
use crate::template::nodes::node_data::NodeData;
use crate::template::nodes::root_data::RootNodeData;
use crate::template::{NodeIndex, Template};
use crate::widgets::tree::Tree;
use crate::Result;
use druid::im::Vector;
use druid_widget_nursery::{selectors, Wedge};
use std::cmp::max;
use std::path::{Path, PathBuf};

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
                Arc::new(|| {
                    Label::dynamic(|data: &NodeData, _env| data.name())
                        // .controller(TemplateUpdate)
                        .boxed()
                }),
                Arc::new(|| {
                    Label::dynamic(|data: &NodeData, _env| {
                        let (add, repl) = data.added_replaced();
                        format!("{}|{}", add, repl)
                    })
                    .align_right()
                    .boxed()
                }),
                Arc::new(|| Label::dynamic(|data: &NodeData, _| data.state_string()).boxed()),
            ],
            NodeData::expanded,
        )
        .lens(TemplateData::root)
        .controller(TemplateUpdate {})
    }

    pub fn node(&self, idx: &NodeIndex) -> &NodeData {
        self.root.node(idx)
    }

    pub fn node_mut(&mut self, idx: &NodeIndex) -> &mut NodeData {
        self.root.node_mut(idx)
    }

    fn update_node(&mut self, event: NodeEvent, idx: NodeIndex) {
        let node = self.node_mut(&idx);
        match event {
            NodeEvent::Path(path_event) => node
                .state
                .path
                .update(path_event, &mut node.cached_path_segment),
            NodeEvent::Site(site_event) => {
                let site = node.ty.site_mut().unwrap();
                site.state.update(site_event, &mut site.history)
            }
        }
    }
}

pub struct TemplateUpdate;

impl<W: Widget<TemplateData>> Controller<TemplateData, W> for TemplateUpdate {
    fn event(
        &mut self,
        child: &mut W,
        ctx: &mut EventCtx,
        event: &Event,
        data: &mut TemplateData,
        env: &Env,
    ) {
        match event {
            Event::Command(cmd) => {
                if let Some(event) = cmd.get(NODE_EVENT) {
                    ctx.set_handled();
                    let (node_event, idx) = event.take().unwrap();
                    data.update_node(node_event, idx);
                    return;
                }
            }
            _ => (),
        }
        child.event(ctx, event, data, env)
    }
}
