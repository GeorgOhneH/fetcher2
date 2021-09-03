use std::convert::TryFrom;
use std::fmt::Display;
use std::sync::Arc;

use druid::kurbo::{BezPath, Size};
use druid::piet::{LineCap, LineJoin, RenderContext, StrokeStyle};
use druid::{theme, WidgetExt, WidgetId};
use druid::widget::{Label, Controller};
use druid::{
    BoxConstraints, Data, Env, Event, EventCtx, LayoutCtx, LifeCycle, LifeCycleCtx, PaintCtx,
    Point, UpdateCtx, Widget, WidgetPod,
};

use druid_widget_nursery::{selectors, Wedge};
use crate::template::node_type::NodeTypeData;
use std::path::PathBuf;
use druid::im::Vector;
use crate::template::nodes::node::MetaData;
use crate::template::nodes::node_data::NodeData;
use crate::widgets::tree::{TreeNode, TreeNodeRoot};


#[derive(Data, Clone, Debug)]
pub struct RootNodeData {
    pub children: Vector<NodeData>,
}


impl TreeNodeRoot<NodeData> for RootNodeData {
    fn children_count(&self) -> usize {
        self.children.len()
    }

    fn get_child(&self, index: usize) -> &NodeData {
        &self.children[index]
    }

    fn for_child_mut(&mut self, index: usize, mut cb: impl FnMut(&mut NodeData, usize)) {
        cb(&mut self.children[index], index);
    }

    fn rm_child(&mut self, index: usize) {
        self.children.remove(index);
    }
}

