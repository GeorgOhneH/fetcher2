use std::convert::TryFrom;
use std::fmt::Display;
use std::path::PathBuf;
use std::sync::Arc;

use druid::im::{HashSet, Vector};
use druid::kurbo::{BezPath, Size};
use druid::piet::{LineCap, LineJoin, RenderContext, StrokeStyle};
use druid::widget::{Controller, Label};
use druid::{theme, Menu, MenuItem, WidgetExt, WidgetId};
use druid::{
    BoxConstraints, Data, Env, Event, EventCtx, LayoutCtx, Lens, LifeCycle, LifeCycleCtx, PaintCtx,
    Point, UpdateCtx, Widget, WidgetPod,
};
use druid_widget_nursery::{selectors, Wedge};
use futures::StreamExt;

use crate::template::communication::{NODE_EVENT, Communication};
use crate::template::node_type::{NodeTypeData, NodeTypeEditData, NodeTypeEditKindData};
use crate::template::nodes::node_data::NodeData;
use crate::widgets::tree::node::{impl_simple_tree_node, TreeNode};
use crate::widgets::tree::NodeIndex;
use crate::TError;
use fetcher2::template::nodes::node::{Node, RawNode};

#[derive(Data, Clone, Debug, Lens)]
pub struct NodeEditData {
    pub expanded: bool,
    pub ty: Option<NodeTypeEditData>,
    pub children: Vector<NodeEditData>,
}

impl_simple_tree_node! {NodeEditData}

impl NodeEditData {
    pub fn new(node: Node<Communication>) -> Self {
        let children: Vector<_> = node
            .children
            .into_iter()
            .map(NodeEditData::new)
            .collect();
        NodeEditData {
            expanded: true,
            children,
            ty: Some(NodeTypeEditData::new(node.ty)),
        }
    }
    pub fn empty(expanded: bool) -> Self {
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
                .insert(idx[0] + offset, NodeEditData::empty(true)),
            _ => self.children[idx[0]].insert_sibling(&idx[1..], offset),
        }
    }

    pub fn insert_child(&mut self, idx: &[usize]) {
        match idx.len() {
            0 => self.children.push_front(NodeEditData::empty(true)),
            _ => self.children[idx[0]].insert_child(&idx[1..]),
        }
    }
}
