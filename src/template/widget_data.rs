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
use crate::background_thread::NEW_TEMPLATE;

#[derive(Debug, Clone, Data, Lens)]
pub struct TemplateData {
    pub root: RootNodeData,
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

    pub fn node_mut(&mut self, idx: &NodeIndex) -> &mut NodeData {
        self.root.node_mut(idx)
    }

    pub fn update_node(&mut self, event: NodeEvent, idx: &NodeIndex) {
        let node = self.node_mut(idx);
        match event {
            NodeEvent::Path(path_event) => node.state.path.update(path_event, &mut node.path),
            NodeEvent::Site(site_event) => {
                let site = node.ty.site_mut().unwrap();
                site.state.update(site_event, &mut site.history)
            }
        }
    }
}

pub struct ContextMenuController;

impl<
        L: Lens<NodeData, bool> + Clone + 'static,
        S: Lens<RootNodeData, Vector<DataNodeIndex>> + Clone + 'static,
        const N: usize,
    > Controller<RootNodeData, Tree<RootNodeData, NodeData, L, S, N>> for ContextMenuController
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
                    node.path.is_some() && data.settings.is_some()
                })
                .on_activate(move |_ctx, data: &mut AppData, _env| {
                    let node = data.template.node(&idx3);
                    let save_path = &data.settings.as_ref().unwrap().downs.save_path;
                    open::that_in_background(save_path.join(node.path.as_ref().unwrap()));
                }),
        )
        .separator()
        .entry(MenuItem::new("Open Website").on_activate(|_ctx, data: &mut AppData, _env| todo!()))
}

