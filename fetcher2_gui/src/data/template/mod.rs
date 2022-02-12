use druid::{Data, Env, Event, EventCtx, Lens, LifeCycle, LifeCycleCtx, Widget};
use druid::{Menu, MenuItem, SingleUse, WidgetExt};
use druid::im::Vector;
use druid::widget::{Controller, Label, WidgetWrapper};
use std::collections::HashSet;
use std::convert::TryInto;
use std::fmt::Debug;
use std::path::PathBuf;
use std::sync::Arc;
use serde::{Deserialize, Serialize};

use crate::controller::{Msg, MSG_THREAD};
use crate::data::AppData;
use crate::data::template::nodes::node::NodeData;
use crate::data::template::nodes::root::RootNodeData;
use crate::widgets::tree::{NodeIndex, Tree};
use crate::widgets::tree::root::TreeNodeRoot;

pub mod node_type;
pub mod nodes;


#[derive(Debug, Clone, Data, Lens, Serialize, Deserialize, Default)]
pub struct TemplateData {
    #[serde(skip)]
    pub root: RootNodeData,

    #[serde(skip)]
    pub save_path: Option<Arc<PathBuf>>,

    #[data(ignore)]
    pub header_sizes: Vec<f64>,
}

impl TemplateData {
    pub fn build_widget() -> impl Widget<TemplateData> {
        Tree::new(
            [
                Label::new("Name").boxed(),
                Label::new("Added|Replaced").align_right().boxed(),
                Label::new("State").boxed(),
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
        .controller(SaveStateController)
    }

    pub fn node(&self, idx: &NodeIndex) -> &NodeData {
        self.root.node(idx)
    }

    pub fn node_mut<V>(&mut self, idx: &NodeIndex, cb: impl FnOnce(&mut NodeData, usize) -> V) -> V {
        self.root.node_mut(idx, cb)
    }
}

pub struct SaveStateController;

impl<L, S, W2, W1, const N: usize> Controller<TemplateData, W2> for SaveStateController
where
    W2: WidgetWrapper<Wrapped = W1> + Widget<TemplateData>,
    W1: WidgetWrapper<Wrapped = Tree<RootNodeData, NodeData, L, S, N>>,
    L: Lens<NodeData, bool> + Clone + 'static,
    S: Lens<RootNodeData, Vector<NodeIndex>> + Clone + 'static,
{
    fn event(
        &mut self,
        child: &mut W2,
        ctx: &mut EventCtx,
        event: &Event,
        data: &mut TemplateData,
        env: &Env,
    ) {
        if let Event::WindowCloseRequested = event {
            data.header_sizes = child.wrapped().wrapped().get_sizes().to_vec()
        }
        child.event(ctx, event, data, env)
    }
    fn lifecycle(
        &mut self,
        child: &mut W2,
        ctx: &mut LifeCycleCtx,
        event: &LifeCycle,
        data: &TemplateData,
        env: &Env,
    ) {
        if let LifeCycle::WidgetAdded = event {
            if let Ok(sizes) = data.header_sizes.clone().try_into() {
                child.wrapped_mut().wrapped_mut().set_sizes(sizes)
            }
        }
        child.lifecycle(ctx, event, data, env)
    }
}

pub struct ContextMenuController;

impl<L, S, const N: usize> Controller<RootNodeData, Tree<RootNodeData, NodeData, L, S, N>>
    for ContextMenuController
where
    L: Lens<NodeData, bool> + Clone + 'static,
    S: Lens<RootNodeData, Vector<NodeIndex>> + Clone + 'static,
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
                    let node : &NodeData = data.node(&idx);
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
    let idx3 = idx;
    Menu::empty()
        .entry(
            MenuItem::new("Run Recursive").on_activate(move |ctx, _data: &mut AppData, _env| {
                ctx.submit_command(
                    MSG_THREAD.with(SingleUse::new(Msg::StartByIndex(indexes.clone()))),
                )
            }),
        )
        .entry(
            MenuItem::new("Run").on_activate(move |ctx, _data: &mut AppData, _env| {
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
        .entry(MenuItem::new("Open Website").on_activate(|_ctx, _data: &mut AppData, _env| todo!()))
}
