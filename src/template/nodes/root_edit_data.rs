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
use crate::template::nodes::node_edit_data::NodeEditData;
use crate::widgets::tree::root::TreeNodeRoot;
use crate::widgets::tree::DataNodeIndex;
use druid::im::Vector;
use druid_widget_nursery::{selectors, Wedge};
use std::path::PathBuf;
use crate::template::nodes::root::RawRootNode;

#[derive(Data, Clone, Debug, Lens)]
pub struct RootNodeEditData {
    pub children: Vector<NodeEditData>,
    pub selected: Vector<DataNodeIndex>,
}

impl RootNodeEditData {
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
}

impl TreeNodeRoot<NodeEditData> for RootNodeEditData {
    fn children_count(&self) -> usize {
        self.children.len()
    }

    fn get_child(&self, index: usize) -> &NodeEditData {
        &self.children[index]
    }

    fn for_child_mut(&mut self, index: usize, mut cb: impl FnMut(&mut NodeEditData, usize)) {
        cb(&mut self.children[index], index);
    }

    fn rm_child(&mut self, index: usize) {
        self.children.remove(index);
    }
}
