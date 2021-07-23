use std::convert::TryFrom;
use std::fmt::Display;
use std::sync::Arc;

use druid::kurbo::{BezPath, Size};
use druid::piet::{LineCap, LineJoin, RenderContext, StrokeStyle};
use druid::widget::{Controller, Label};
use druid::{theme, WidgetExt, WidgetId};
use druid::{
    BoxConstraints, Data, Env, Event, EventCtx, LayoutCtx, LifeCycle, LifeCycleCtx, PaintCtx,
    Point, UpdateCtx, Widget, WidgetPod,
};

use crate::template::communication::NODE_EVENT;
use crate::template::node_type::site::SiteState;
use crate::template::node_type::site::{
    DownloadEvent, LoginEvent, RunEvent, SiteEvent, UrlFetchEvent,
};
use crate::template::node_type::NodeTypeData;
use crate::template::nodes::node::{NodeEvent, PathEvent};
use crate::template::MetaData;
use crate::TError;
use druid::im::Vector;
use druid_widget_nursery::{selectors, Wedge};
use std::path::PathBuf;
use futures::StreamExt;

#[derive(Data, Clone, Debug)]
pub struct NodeData {
    pub ty: NodeTypeData,
    pub meta_data: MetaData,
    pub children: Vector<NodeData>,

    #[data(same_fn = "PartialEq::eq")]
    pub cached_path: Option<PathBuf>,

    pub state: NodeState,
}

impl NodeData {
    fn name(&self) -> String {
        if let Some(cache_path) = self.cached_path.as_ref() {
            cache_path
                .file_name()
                .map(|os_str| os_str.to_string_lossy().to_string())
                .unwrap_or("Root".to_owned())
        } else {
            self.ty.name()
        }
    }

    fn added_replaced(&self) -> (usize, usize) {
        match &self.ty {
            NodeTypeData::Site(site) => site.added_replaced(),
            NodeTypeData::Folder(_) => self.added_replaced_folder(),
        }
    }

    fn added_replaced_folder(&self) -> (usize, usize) {
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

    fn state_string(&self) -> String {
        match &self.ty {
            NodeTypeData::Folder(_) => "".to_string(),
            NodeTypeData::Site(site) => {
                match self.state.current_state() {
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
                        }
                    }
                }
            }
        }
    }

    pub fn widget(num: usize) -> Box<dyn Widget<Self>> {
        match num {
            0 => Label::dynamic(|data: &NodeData, _env| data.name())
                .controller(TemplateUpdate)
                .boxed(),
            1 => Label::dynamic(|data: &NodeData, _env| {
                let (add, repl) = data.added_replaced();
                format!("{}|{}", add, repl)
            })
            .align_right()
            .boxed(),
            2 => Label::dynamic(|data: &NodeData, _| data.state_string()).boxed(),
            _ => panic!("Not implemented"),
        }
    }
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

struct TemplateUpdate;

impl<W: Widget<NodeData>> Controller<NodeData, W> for TemplateUpdate {
    fn event(
        &mut self,
        child: &mut W,
        ctx: &mut EventCtx,
        event: &Event,
        data: &mut NodeData,
        env: &Env,
    ) {
        if let Event::Command(cmd) = event {
            if let Some(event) = cmd.get(NODE_EVENT) {
                match event.take().unwrap() {
                    NodeEvent::Path(path_event) => {
                        data.state.path.update(path_event, &mut data.cached_path)
                    }
                    NodeEvent::Site(site_event) => {
                        let site = data.ty.site_mut().unwrap();
                        site.state.update(site_event, &mut site.history)
                    }
                }
                return;
            }
        }
        child.event(ctx, event, data, env)
    }
}
