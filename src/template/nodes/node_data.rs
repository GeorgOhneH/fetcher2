use std::convert::TryFrom;
use std::fmt::Display;
use std::sync::Arc;

use druid::kurbo::{BezPath, Size};
use druid::piet::{LineCap, LineJoin, RenderContext, StrokeStyle};
use druid::widget::{Controller, Label};
use druid::{theme, Menu, WidgetExt, WidgetId, MenuItem};
use druid::{
    BoxConstraints, Data, Env, Event, EventCtx, LayoutCtx, LifeCycle, LifeCycleCtx, PaintCtx,
    Point, UpdateCtx, Widget, WidgetPod, Lens,
};

use crate::template::communication::NODE_EVENT;
use crate::template::node_type::site::SiteState;
use crate::template::node_type::site::{
    DownloadEvent, LoginEvent, RunEvent, SiteEvent, UrlFetchEvent,
};
use crate::template::node_type::NodeTypeData;
use crate::template::nodes::node::{NodeEvent, PathEvent};
use crate::template::{MetaData, NodeIndex};
use crate::{AppData, TError};
use druid::im::Vector;
use druid_widget_nursery::{selectors, Wedge};
use futures::StreamExt;
use std::path::PathBuf;
use crate::widgets::tree::TreeNode;

#[derive(Data, Clone, Debug, Lens)]
pub struct NodeData {
    pub expanded: bool,
    pub ty: NodeTypeData,
    pub meta_data: MetaData,
    pub children: Vector<NodeData>,

    #[data(same_fn = "PartialEq::eq")]
    pub cached_path_segment: Option<PathBuf>,

    pub state: NodeState,
}

impl NodeData {
    pub fn node(&self, idx: &[usize]) -> &NodeData {
        if idx.len() == 0 {
            self
        } else {
            self.children[idx[0]].node(&idx[1..])
        }
    }

    pub fn node_mut(&mut self, idx: &[usize]) -> &mut NodeData {
        if idx.len() == 0 {
            self
        } else {
            self.children[idx[0]].node_mut(&idx[1..])
        }
    }

    pub fn name(&self) -> String {
        if let Some(cache_path) = self.cached_path_segment.as_ref() {
            cache_path
                .file_name()
                .map(|os_str| os_str.to_string_lossy().to_string())
                .unwrap_or("Root".to_owned())
        } else {
            self.ty.name()
        }
    }

    pub fn added_replaced(&self) -> (usize, usize) {
        match &self.ty {
            NodeTypeData::Site(site) => site.added_replaced(),
            NodeTypeData::Folder(_) => self.added_replaced_folder(),
        }
    }

    pub fn added_replaced_folder(&self) -> (usize, usize) {
        let init = match &self.ty {
            NodeTypeData::Site(site) => site.added_replaced(),
            NodeTypeData::Folder(_) => (0, 0),
        };
        self.children
            .iter()
            .map(Self::added_replaced_folder)
            .fold(init, |(acc_add, acc_repl), (add, repl)| {
                (acc_add + add, acc_repl + repl)
            })
    }

    pub fn state_string(&self) -> String {
        match &self.ty {
            NodeTypeData::Folder(_) => "".to_string(),
            NodeTypeData::Site(site) => match self.state.current_state() {
                CurrentState::Active => "Calculation Path".to_string(),
                CurrentState::Error => "Error while calculation Path".to_string(),
                CurrentState::Idle => match site.state.run {
                    0 => "Idle".to_string(),
                    _ => match site.state.login.current_state() {
                        CurrentState::Active => "Logging in".to_string(),
                        CurrentState::Error => "Login while logging in".to_string(),
                        CurrentState::Idle => match site.state.fetch.current_state() {
                            CurrentState::Active => "Fetching Urls".to_string(),
                            CurrentState::Error => "Error while fetching Urls".to_string(),
                            CurrentState::Idle => site.state.download.state_string(),
                        },
                    },
                },
            },
        }
    }
}

impl TreeNode for NodeData {
    fn children_count(&self) -> usize {
        self.children.len()
    }

    fn get_child(&self, index: usize) -> &Self {
        &self.children[index]
    }

    fn for_child_mut(&mut self, index: usize, mut cb: impl FnMut(&mut Self, usize)) {
        cb(&mut self.children[index], index);
    }

    fn rm_child(&mut self, index: usize) {
        self.children.remove(index);
    }
}
// pub struct ContextMenuController;
//
// impl<W: Widget<AppData>> Controller<AppData, W> for ContextMenuController {
//     fn event(
//         &mut self,
//         child: &mut W,
//         ctx: &mut EventCtx,
//         event: &Event,
//         data: &mut AppData,
//         env: &Env,
//     ) {
//         match event {
//             Event::MouseDown(ref mouse) if mouse.button.is_right() => {
//                 ctx.show_context_menu(make_node_menu(), mouse.pos);
//             }
//             _ => child.event(ctx, event, data, env),
//         }
//     }
// }

fn make_node_menu() -> Menu<AppData> {
    Menu::empty()
        .entry(
            MenuItem::new("Hello2")
                .on_activate(|_ctx, data: &mut AppData, _env| {  }),
        )
}

pub enum CurrentState {
    Idle,
    Active,
    Error,
}

#[derive(Data, Clone, Debug)]
pub struct NodeState {
    pub path: PathState,
}

impl NodeState {
    pub fn new() -> Self {
        Self {
            path: PathState::new(),
        }
    }

    pub fn current_state(&self) -> CurrentState {
        if self.path.count != 0 {
            CurrentState::Active
        } else if self.path.errs.len() != 0 {
            CurrentState::Error
        } else {
            CurrentState::Idle
        }
    }
}

#[derive(Data, Clone, Debug)]
pub struct PathState {
    count: usize,
    errs: Vector<Arc<TError>>,
}

impl PathState {
    pub fn new() -> Self {
        Self {
            count: 0,
            errs: Vector::new(),
        }
    }

    pub fn update(&mut self, event: PathEvent, cache_path: &mut Option<PathBuf>) {
        match event {
            PathEvent::Start => {
                self.count += 1;
            }
            PathEvent::Finish(path) => {
                *cache_path = Some(path);
                self.count -= 1;
            }
            PathEvent::Err(err) => {
                self.count -= 1;
                self.errs.push_back(Arc::new(err));
            }
            PathEvent::Cached(path) => {
                *cache_path = Some(path);
            }
        }
    }
}

