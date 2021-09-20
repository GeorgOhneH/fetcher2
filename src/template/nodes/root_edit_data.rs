use std::convert::TryFrom;
use std::fmt::Display;
use std::path::PathBuf;
use std::sync::Arc;

use druid::{theme, WidgetExt, WidgetId};
use druid::{
    BoxConstraints, Data, Env, Event, EventCtx, LayoutCtx, Lens, LifeCycle, LifeCycleCtx, PaintCtx,
    Point, UpdateCtx, Widget, WidgetPod,
};
use druid::im::Vector;
use druid::kurbo::{BezPath, Size};
use druid::piet::{LineCap, LineJoin, RenderContext, StrokeStyle};
use druid::widget::{Controller, Label};
use druid_widget_nursery::{selectors, Wedge};

use crate::edit_window::NodePosition;
use crate::template::node_type::NodeTypeData;
use crate::template::nodes::node::{MetaData, RawNode};
use crate::template::nodes::node_data::NodeData;
use crate::template::nodes::node_edit_data::NodeEditData;
use crate::template::nodes::root::RawRootNode;
use crate::widgets::tree::DataNodeIndex;
use crate::widgets::tree::root::{impl_simple_tree_root, TreeNodeRoot};

#[derive(Data, Clone, Debug, Lens)]
pub struct RootNodeEditData {
    pub children: Vector<NodeEditData>,
    pub selected: Vector<DataNodeIndex>,
}


impl_simple_tree_root!{RootNodeEditData, NodeEditData}

impl RootNodeEditData {
    pub fn new() -> Self {
        Self {
            children: Vector::new(),
            selected: Vector::new(),
        }
    }

    pub fn raw(self) -> RawRootNode {
        let children = self
            .children
            .into_iter()
            .filter_map(|child| child.raw())
            .collect();
        RawRootNode { children }
    }

    pub fn node(&self, idx: &[usize]) -> &NodeEditData {
        if idx.len() == 0 {
            panic!("Can't access root node")
        } else {
            self.children[idx[0]].node(&idx[1..])
        }
    }

    pub fn node_mut(&mut self, idx: &[usize]) -> &mut NodeEditData {
        if idx.len() == 0 {
            panic!("Can't access root node")
        } else {
            self.children[idx[0]].node_mut(&idx[1..])
        }
    }

    pub fn remove(&mut self, idx: &[usize]) -> NodeEditData {
        match idx.len() {
            0 => panic!("Can't remove the root node"),
            1 => self.children.remove(idx[0]),
            _ => self.children[idx[0]].remove(&idx[1..]),
        }
    }

    pub fn insert_node(&mut self, idx: &[usize], pos: NodePosition) {
        match pos {
            NodePosition::Child => self.insert_child(idx),
            NodePosition::Above => self.insert_sibling(idx, 0),
            NodePosition::Below => self.insert_sibling(idx, 1),
        }
    }

    pub fn insert_sibling(&mut self, idx: &[usize], offset: usize) {
        match idx.len() {
            0 => panic!("Can't do this"),
            1 => self
                .children
                .insert(idx[0] + offset, NodeEditData::new(true)),
            _ => self.children[idx[0]].insert_sibling(&idx[1..], offset),
        }
    }

    pub fn insert_child(&mut self, idx: &[usize]) {
        match idx.len() {
            0 => self.children.push_back(NodeEditData::new(true)),
            _ => self.children[idx[0]].insert_child(&idx[1..]),
        }
    }
}
