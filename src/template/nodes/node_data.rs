use std::convert::TryFrom;
use std::fmt::Display;
use std::sync::Arc;

use druid::kurbo::{BezPath, Size};
use druid::piet::{LineCap, LineJoin, RenderContext, StrokeStyle};
use druid::widget::{Controller, Label};
use druid::{theme, Menu, MenuItem, WidgetExt, WidgetId};
use druid::{
    BoxConstraints, Data, Env, Event, EventCtx, LayoutCtx, Lens, LifeCycle, LifeCycleCtx, PaintCtx,
    Point, UpdateCtx, Widget, WidgetPod,
};

use crate::template::communication::NODE_EVENT;
use crate::template::node_type::NodeTypeData;
use crate::template::nodes::node::{NodeEvent, PathEvent};
use crate::template::MetaData;
use crate::widgets::tree::node::TreeNode;
use crate::widgets::tree::NodeIndex;
use crate::{AppData, TError};
use druid::im::{HashSet, Vector};
use druid_widget_nursery::{selectors, Wedge};
use futures::StreamExt;
use std::path::PathBuf;

#[derive(Data, Clone, Debug, Lens)]
pub struct NodeData {
    pub expanded: bool,
    pub ty: NodeTypeData,
    pub meta_data: MetaData,
    pub children: Vector<NodeData>,

    #[data(same_fn = "PartialEq::eq")]
    pub cached_path_segment: Option<PathBuf>,
    #[data(same_fn = "PartialEq::eq")]
    pub path: Option<PathBuf>,

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

    pub fn child_indexes(
        &self,
        current_idx: NodeIndex,
        set: &mut std::collections::HashSet<NodeIndex>,
    ) {
        for (i, child) in self.children.iter().enumerate() {
            let mut child_idx = current_idx.clone();
            child_idx.push(i);
            child.child_indexes(child_idx, set);
        }
        set.insert(current_idx);
    }

    pub fn name(&self) -> String {
        if let Some(path) = &self.path {
            path.file_name()
                .map(|os_str| os_str.to_string_lossy().to_string())
                .unwrap_or("Root".to_owned())
        } else if let Some(cache_path) = self.cached_path_segment.as_ref() {
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
                CurrentState::Idle => match site.state.login.current_state() {
                    CurrentState::Active => "Logging in".to_string(),
                    CurrentState::Error => "Error while logging in".to_string(),
                    CurrentState::Idle => match site.state.fetch.current_state() {
                        CurrentState::Active => "Fetching Urls".to_string(),
                        CurrentState::Error => "Error while fetching Urls".to_string(),
                        CurrentState::Idle => match site.state.run {
                            0 => site.state.download.state_string(),
                            _ => "Cleaning Up".to_string(),
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

    pub fn update(&mut self, event: PathEvent, path: &mut Option<PathBuf>) {
        match event {
            PathEvent::Start => {
                self.count += 1;
            }
            PathEvent::Finish(new_path) => {
                *path = Some(new_path);
                self.count -= 1;
            }
            PathEvent::Err(err) => {
                self.count -= 1;
                dbg!(&err);
                self.errs.push_back(Arc::new(err));
            }
            PathEvent::Cached(new_path) => {
                *path = Some(new_path);
            }
        }
    }
}
