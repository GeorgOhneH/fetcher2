use druid::im::Vector;
use druid::{Data, Lens};
use fetcher2::template::nodes::node::{Node, NodeEventKind, PathEventKind};
use fetcher2::TError;
use std::borrow::Cow;
use std::path::PathBuf;
use std::sync::Arc;

use crate::data::template::node_type::NodeTypeData;
use crate::widgets::tree::node::TreeNode;
use crate::widgets::tree::NodeIndex;

#[derive(Data, Clone, Debug, Lens, TreeNode)]
pub struct NodeData {
    pub expanded: bool,
    pub ty: NodeTypeData,
    pub children: Vector<NodeData>,

    #[data(same_fn = "PartialEq::eq")]
    pub cached_path_segment: Option<PathBuf>,
    #[data(same_fn = "PartialEq::eq")]
    pub path: Option<PathBuf>,

    pub state: NodeState,
}

impl NodeData {
    pub fn new(node: &Node) -> Self {
        let children: Vector<_> = node.children.iter().map(NodeData::new).collect();
        Self {
            expanded: true,
            children,
            cached_path_segment: node.cached_path_segment.clone(),
            ty: NodeTypeData::new(&node.ty),
            state: NodeState::new(),
            path: None,
        }
    }
    pub fn child_indexes(
        &self,
        current_idx: NodeIndex,
        set: &mut std::collections::HashSet<NodeIndex>,
    ) {
        for (i, child) in self.children.iter().enumerate() {
            let mut child_idx = current_idx.clone();
            child_idx.push_back(i);
            child.child_indexes(child_idx, set);
        }
        set.insert(current_idx);
    }

    pub fn name(&self) -> String {
        if let Some(path) = &self.path {
            path.file_name()
                .map(|os_str| os_str.to_string_lossy().to_string())
                .unwrap_or_else(|| "Root".to_owned())
        } else if let Some(cache_path) = self.cached_path_segment.as_ref() {
            cache_path
                .file_name()
                .map(|os_str| os_str.to_string_lossy().to_string())
                .unwrap_or_else(|| "Root".to_owned())
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
            NodeTypeData::Site(site) => {
                let order = [
                    self.state.current_state(),
                    site.state.login.current_state(),
                    site.state.fetch.current_state(),
                    site.state.download.current_state(),
                    site.state.run_state(),
                ];

                for state in &order {
                    if let CurrentState::Error(msg) = state {
                        return msg.to_string();
                    }
                }

                if self.state.canceled {
                    return "Canceled".to_string();
                }

                for state in &order {
                    if let CurrentState::Active(msg) = state {
                        return msg.to_string();
                    }
                }
                "Idle".to_string()
            }
        }
    }

    pub fn update_node(&mut self, event: NodeEventKind) {
        match event {
            NodeEventKind::Path(path_event) => {
                if path_event.is_start() {
                    self.state.canceled = false;
                    self.state.reset();
                }
                self.state.path.update(path_event, &mut self.path)
            }
            NodeEventKind::Site(site_event) => {
                let site = self.ty.site_mut().unwrap();
                if site_event.is_start() {
                    self.state.canceled = false;
                    site.state.reset();
                }
                site.state.update(site_event, &mut site.history)
            }
            NodeEventKind::Canceled => {
                if self.state.path.count != 0 || !self.ty.is_finished() {
                    self.state.canceled = true;
                }
            }
        }
    }
}

pub enum CurrentState<'a> {
    Idle,
    Active(Cow<'a, str>),
    Error(Cow<'a, str>),
}

#[derive(Data, Clone, Debug)]
pub struct NodeState {
    pub path: PathState,
    pub canceled: bool,
}

impl NodeState {
    pub fn new() -> Self {
        Self {
            path: PathState::new(),
            canceled: false,
        }
    }

    pub fn current_state(&self) -> CurrentState {
        if self.path.count != 0 {
            CurrentState::Active("Calculating Path".into())
        } else if !self.path.errs.is_empty() {
            CurrentState::Error("Error while calculating Path".into())
        } else {
            CurrentState::Idle
        }
    }

    pub fn reset(&mut self) {
        self.path.reset();
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

    pub fn reset(&mut self) {
        self.count = 0;
        self.errs.clear();
    }

    pub fn update(&mut self, event: PathEventKind, path: &mut Option<PathBuf>) {
        match event {
            PathEventKind::Start => {
                self.count += 1;
            }
            PathEventKind::Finish(new_path) => {
                *path = Some(new_path);
                self.count -= 1;
            }
            PathEventKind::Err(err) => {
                self.count -= 1;
                dbg!(&err);
                self.errs.push_back(Arc::new(err));
            }
            PathEventKind::Cached(new_path) => {
                *path = Some(new_path);
            }
        }
    }
}
