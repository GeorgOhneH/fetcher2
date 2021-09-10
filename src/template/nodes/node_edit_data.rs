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
use crate::template::node_type::{NodeTypeData, NodeTypeEditData};
use crate::template::nodes::node::{NodeEvent, PathEvent};
use crate::template::MetaData;
use crate::widgets::tree::node::TreeNode;
use crate::widgets::tree::NodeIndex;
use crate::{AppData, TError};
use druid::im::{HashSet, Vector};
use druid_widget_nursery::{selectors, Wedge};
use futures::StreamExt;
use std::path::PathBuf;
use crate::template::nodes::node_data::NodeData;

#[derive(Data, Clone, Debug, Lens)]
pub struct NodeEditData {
    pub expanded: bool,
    pub ty: NodeTypeEditData,
    pub meta_data: MetaData,
    pub children: Vector<NodeEditData>,

}

impl NodeEditData {
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
}

impl TreeNode for NodeEditData {
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
