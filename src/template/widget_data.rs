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

use crate::{Result};
use crate::background_thread::NEW_TEMPLATE;
use crate::controller::{Msg, MSG_THREAD};
use crate::template::communication::NODE_EVENT;
use crate::template::nodes::node::NodeEvent;
use crate::template::nodes::node_data::NodeData;
use crate::template::nodes::root_data::RootNodeData;
use crate::template::Template;
use crate::widgets::tree::{DataNodeIndex, NodeIndex, Tree};
use crate::data::AppData;
use crate::widgets::tree::root::TreeNodeRoot;

#[derive(Debug, Clone, Data, Lens)]
pub struct TemplateData {
    pub root: RootNodeData,
    #[data(eq)]
    pub save_path: Option<PathBuf>,
}

impl TemplateData {
    pub fn new() -> Self {
        Self {
            root: RootNodeData::new(),
            save_path: None,
        }
    }
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
                Arc::new(|| Label::dynamic(|data: &NodeData, _env| data.name()).boxed()),
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
            RootNodeData::selected,
        )
        .controller(ContextMenuController {})
        .lens(TemplateData::root)
    }

    pub fn node(&self, idx: &NodeIndex) -> &NodeData {
        self.root.node(idx)
    }

    pub fn node_mut<V>(&mut self, idx: &NodeIndex, cb: impl FnOnce(&mut NodeData, usize) -> V) -> V {
        self.root.node_mut(idx, cb)
    }
}

pub struct ContextMenuController;

impl<L, S, const N: usize> Controller<RootNodeData, Tree<RootNodeData, NodeData, L, S, N>>
    for ContextMenuController
where
    L: Lens<NodeData, bool> + Clone + 'static,
    S: Lens<RootNodeData, Vector<DataNodeIndex>> + Clone + 'static,
{
    fn event(
        &mut self,
        child: &mut Tree<RootNodeData, NodeData, L, S, N>,
        ctx: &mut EventCtx,
        event: &Event,
        data: &mut RootNodeData,
        env: &Env,
    ) {
        match event {
            Event::MouseDown(ref mouse) if mouse.button.is_right() => {
                if let Some(idx) = child.node_at(mouse.pos) {
                    let node = data.node(&idx);
                    let mut indexes = HashSet::new();
                    node.child_indexes(idx.clone(), &mut indexes);
                    ctx.show_context_menu(make_node_menu(idx, indexes), mouse.window_pos);
                    return;
                }
            }
            _ => (),
        }
        child.event(ctx, event, data, env)
    }
}

fn make_node_menu(idx: NodeIndex, indexes: HashSet<NodeIndex>) -> Menu<AppData> {
    let idx1 = idx.clone();
    let idx2 = idx.clone();
    let idx3 = idx.clone();
    Menu::empty()
        .entry(
            MenuItem::new("Run Recursive").on_activate(move |ctx, data: &mut AppData, _env| {
                ctx.submit_command(
                    MSG_THREAD.with(SingleUse::new(Msg::StartByIndex(indexes.clone()))),
                )
            }),
        )
        .entry(
            MenuItem::new("Run").on_activate(move |ctx, data: &mut AppData, _env| {
                let mut set = HashSet::with_capacity(1);
                set.insert(idx1.clone());
                ctx.submit_command(MSG_THREAD.with(SingleUse::new(Msg::StartByIndex(set))))
            }),
        )
        .separator()
        .entry(
            MenuItem::new("Open Folder")
                .enabled_if(move |data: &AppData, _env| {
                    let node = data.template.node(&idx2);
                    node.path.is_some() && data.get_settings().is_some()
                })
                .on_activate(move |_ctx, data: &mut AppData, _env| {
                    let node = data.template.node(&idx3);
                    let save_path = &data.get_settings().as_ref().unwrap().download.save_path;
                    open::that_in_background(save_path.join(node.path.as_ref().unwrap()));
                }),
        )
        .separator()
        .entry(MenuItem::new("Open Website").on_activate(|_ctx, data: &mut AppData, _env| todo!()))
}
