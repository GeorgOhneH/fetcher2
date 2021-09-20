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

use crate::template::node_type::NodeTypeData;
use crate::template::nodes::node::MetaData;
use crate::template::nodes::node_data::NodeData;
use crate::widgets::tree::DataNodeIndex;
use crate::widgets::tree::root::{impl_simple_tree_root, TreeNodeRoot};

#[derive(Data, Clone, Debug, Lens)]
pub struct RootNodeData {
    pub children: Vector<NodeData>,
    pub selected: Vector<DataNodeIndex>,
}

impl_simple_tree_root!{RootNodeData, NodeData}

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

