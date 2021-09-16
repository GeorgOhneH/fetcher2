use std::convert::TryFrom;
use std::fmt::Display;
use std::path::PathBuf;
use std::sync::Arc;

use druid::{Menu, MenuItem, theme, WidgetExt, WidgetId};
use druid::{
    BoxConstraints, Data, Env, Event, EventCtx, LayoutCtx, Lens, LifeCycle, LifeCycleCtx, PaintCtx,
    Point, UpdateCtx, Widget, WidgetPod,
};
use druid::im::{HashSet, Vector};
use druid::kurbo::{BezPath, Size};
use druid::piet::{LineCap, LineJoin, RenderContext, StrokeStyle};
use druid::widget::{Controller, Label};
use druid_widget_nursery::{selectors, Wedge};
use futures::StreamExt;

use crate::{AppData, TError};
use crate::template::communication::NODE_EVENT;
use crate::template::MetaData;
use crate::template::node_type::{NodeTypeData, NodeTypeEditData, NodeTypeEditKindData};
use crate::template::nodes::node::{NodeEvent, PathEvent, RawNode};
use crate::template::nodes::node_data::NodeData;
use crate::widgets::tree::node::TreeNode;
use crate::widgets::tree::NodeIndex;

#[derive(Data, Clone, Debug, Lens)]
pub struct NodeEditData {
    pub expanded: bool,
    pub ty: Option<NodeTypeEditData>,
    pub children: Vector<NodeEditData>,
}

impl NodeEditData {
    pub fn new(expanded: bool) -> Self {
        Self {
            expanded,
            ty: None,
            children: Vector::new(),
        }
    }

    pub fn raw(self) -> Option<RawNode> {
        if let Some(ty) = self.ty {
            let children = self
                .children
                .into_iter()
                .filter_map(|child| child.raw())
                .collect();
            Some(RawNode {
                ty: ty.kind.raw(),
                children,
                meta_data: ty.meta_data,
                cached_path_segment: None,
            })
        } else {
            None
        }
    }
    pub fn name(&self) -> String {
        if let Some(ty) = &self.ty {
            ty.kind.name()
        } else {
            "New Node".to_string()
        }
    }
    pub fn node(&self, idx: &[usize]) -> &Self {
        if idx.len() == 0 {
            self
        } else {
            self.children[idx[0]].node(&idx[1..])
        }
    }

    pub fn node_mut(&mut self, idx: &[usize]) -> &mut Self {
        if idx.len() == 0 {
            self
        } else {
            self.children[idx[0]].node_mut(&idx[1..])
        }
    }

    pub fn remove(&mut self, idx: &[usize]) -> NodeEditData {
        match idx.len() {
            0 => unreachable!(),
            1 => self.children.remove(idx[0]),
            _ => self.children[idx[0]].remove(&idx[1..]),
        }
    }

    pub fn insert_sibling(&mut self, idx: &[usize], offset: usize) {
        match idx.len() {
            0 => unreachable!(),
            1 => self
                .children
                .insert(idx[0] + offset, NodeEditData::new(true)),
            _ => self.children[idx[0]].insert_sibling(&idx[1..], offset),
        }
    }

    pub fn insert_child(&mut self, idx: &[usize]) {
        match idx.len() {
            0 => self.children.push_front(NodeEditData::new(true)),
            _ => self.children[idx[0]].insert_child(&idx[1..]),
        }
    }
}

impl TreeNode for NodeEditData {
    fn children_count(&self) -> usize {
        self.children.len()
    }

    fn get_child(&self, index: usize) -> &Self {
        &self.children[index]
    }

    fn for_child_mut(&mut self, index: usize, mut cb: impl FnMut(&mut Self, usize)) {
        let mut new_child = self.children[index].to_owned();
        cb(&mut new_child, index);
        if !new_child.same(&self.children[index]) {
            self.children[index] = new_child;
        }
    }

    fn rm_child(&mut self, index: usize) {
        self.children.remove(index);
    }
}
