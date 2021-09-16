use std::convert::TryFrom;
use std::fmt::Display;
use std::sync::Arc;

use druid::kurbo::{BezPath, Size};
use druid::piet::{LineCap, LineJoin, RenderContext, StrokeStyle};
use druid::widget::{Controller, Label};
use druid::{theme, WidgetExt, WidgetId};
use druid::{
    BoxConstraints, Data, Env, Event, EventCtx, LayoutCtx, Lens, LifeCycle, LifeCycleCtx, PaintCtx,
    Point, UpdateCtx, Widget, WidgetPod,
};

use crate::template::node_type::NodeTypeData;
use crate::template::nodes::node::MetaData;
use crate::template::nodes::node_data::NodeData;
use crate::widgets::tree::root::TreeNodeRoot;
use crate::widgets::tree::DataNodeIndex;
use druid::im::Vector;
use druid_widget_nursery::{selectors, Wedge};
use std::path::PathBuf;

#[derive(Data, Clone, Debug, Lens)]
pub struct RootNodeData {
    pub children: Vector<NodeData>,
    pub selected: Vector<DataNodeIndex>,
}

impl RootNodeData {
    pub fn new() -> Self {
        Self {
            children: Vector::new(),
            selected: Vector::new(),
        }
    }
    pub fn node(&self, idx: &[usize]) -> &NodeData {
        if idx.len() == 0 {
            panic!("Can't access root node")
        } else {
            self.children[idx[0]].node(&idx[1..])
        }
    }

    pub fn node_mut(&mut self, idx: &[usize]) -> &mut NodeData {
        if idx.len() == 0 {
            panic!("Can't access root node")
        } else {
            self.children[idx[0]].node_mut(&idx[1..])
        }
    }
}

impl TreeNodeRoot<NodeData> for RootNodeData {
    fn children_count(&self) -> usize {
        self.children.len()
    }

    fn get_child(&self, index: usize) -> &NodeData {
        &self.children[index]
    }

    fn for_child_mut(&mut self, index: usize, mut cb: impl FnMut(&mut NodeData, usize)) {
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
