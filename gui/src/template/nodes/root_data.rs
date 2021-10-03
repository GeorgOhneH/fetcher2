use std::convert::TryFrom;
use std::fmt::Display;
use std::path::PathBuf;
use std::sync::Arc;

use druid::im::Vector;
use druid::kurbo::{BezPath, Size};
use druid::piet::{LineCap, LineJoin, RenderContext, StrokeStyle};
use druid::widget::{Controller, Label};
use druid::{theme, WidgetExt, WidgetId};
use druid::{
    BoxConstraints, Data, Env, Event, EventCtx, LayoutCtx, Lens, LifeCycle, LifeCycleCtx, PaintCtx,
    Point, UpdateCtx, Widget, WidgetPod,
};
use druid_widget_nursery::{selectors, Wedge};

use crate::template::node_type::NodeTypeData;
use crate::template::nodes::node_data::NodeData;
use crate::widgets::tree::root::{impl_simple_tree_root, TreeNodeRoot};
use crate::widgets::tree::DataNodeIndex;
use fetcher2::template::nodes::root::RootNode;
use crate::template::communication::Communication;

#[derive(Data, Clone, Debug, Lens)]
pub struct RootNodeData {
    pub children: Vector<NodeData>,
    pub selected: Vector<DataNodeIndex>,
}

impl_simple_tree_root! {RootNodeData, NodeData}

impl RootNodeData {
    pub fn new(root: RootNode<Communication>) -> Self {
        let children: Vector<_> = root.children.into_iter().map(NodeData::new).collect();

        Self {
            children,
            selected: Vector::new(),
        }
    }
    pub fn empty() -> Self {
        Self {
            children: Vector::new(),
            selected: Vector::new(),
        }
    }
}
