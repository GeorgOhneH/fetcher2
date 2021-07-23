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
use crate::template::nodes::node_widget::{NodeWidget};
use crate::template::nodes::node_data::NodeData;


#[derive(Data, Clone, Debug)]
pub struct RootNodeData {
    pub children: Vector<NodeData>,
}


/// An internal widget used to display a single node and its children
/// This is used recursively to build the tree.
pub struct RootNodeWidget
{
    children: Vec<WidgetPod<NodeData, NodeWidget>>,
    section_bc: Option<Vec<(f64, f64)>>,
    section_num: usize,
}

impl RootNodeWidget {
    /// Create a TreeNodeWidget from a TreeNode.
    pub fn new() -> Self {
        RootNodeWidget {
            children: Vec::new(),
            section_bc: None,
            section_num: 0,
        }
    }

    pub fn add_child(&mut self, mut child: NodeWidget) {
        child.index = self.children.len();
        self.children.push(WidgetPod::new(child));
    }

    pub fn add_children(&mut self, children: Vec<NodeWidget>) {
        for child in children.into_iter() {
            self.add_child(child)
        }
    }

    pub fn set_section_bc(&mut self, section_bc: Vec<(f64, f64)>) {
        self.section_bc = Some(section_bc)
    }

    pub fn set_section_num(&mut self, section_num: usize) {
        self.section_num = section_num;
        for widget in self.children.iter_mut() {
            widget.widget_mut().set_section_num(section_num)
        }
    }

    pub fn updated_selection(&mut self, selection: &[usize]) {
        if selection.is_empty() {
            panic!("Should never be empty")
        }
        let idx = selection[0];
        let child_selection = &selection[1..];
        for (i, widget) in self.children.iter_mut().enumerate() {
            if idx == i {
                widget.widget_mut().updated_selection(child_selection)
            } else {
                widget.widget_mut().unselect()
            }
        }
    }

}

impl Widget<RootNodeData> for RootNodeWidget
{
    fn event(&mut self, ctx: &mut EventCtx, event: &Event, data: &mut RootNodeData, env: &Env) {
        for (child_widget_node, child_data) in self.children.iter_mut().zip(data.children.iter_mut()) {
            child_widget_node.event(ctx, event, child_data, env);
        }
    }

    fn lifecycle(&mut self, ctx: &mut LifeCycleCtx, event: &LifeCycle, data: &RootNodeData, env: &Env) {
        for (child_widget_node, child_data) in self.children.iter_mut().zip(data.children.iter()) {
            child_widget_node.lifecycle(ctx, event, child_data, env);
        }
    }

    fn update(&mut self, ctx: &mut UpdateCtx, old_data: &RootNodeData, data: &RootNodeData, env: &Env) {
        if !old_data.same(data) {
            for (child_widget_node, child_data) in self.children.iter_mut().zip(data.children.iter()) {
                child_widget_node.update(ctx, child_data, env);
            }
            ctx.request_layout();
            ctx.children_changed();
        }
    }

    fn layout(&mut self, ctx: &mut LayoutCtx, bc: &BoxConstraints, data: &RootNodeData, env: &Env) -> Size {
        let mut min_width = bc.min().width;
        let mut max_width = bc.max().width;
        let mut size = Size::new(0., 0.);

        for (child_widget_node, child_data) in self.children.iter_mut().zip(data.children.iter()) {
            let child_bc = BoxConstraints::new(
                Size::new(min_width, 0.0),
                Size::new(max_width, f64::INFINITY),
            );

            child_widget_node.widget_mut().set_section_bc(self.section_bc.as_ref().unwrap().clone());
            let child_size = child_widget_node.layout(ctx, &child_bc, child_data, env);
            let child_pos = Point::new(0., size.height); // We position the child at the current height
            child_widget_node.set_origin(ctx, child_data, env, child_pos);
            size.height += child_size.height; // Increment the height of this node by the height of this child node
            if child_size.width > size.width {
                size.width = child_size.width;
            }
        }
        bc.constrain(size)
    }

    fn paint(&mut self, ctx: &mut PaintCtx, data: &RootNodeData, env: &Env) {
        for (child_widget_node, child_data) in self.children.iter_mut().zip(data.children.iter()) {
            child_widget_node.paint(ctx, child_data, env);
        }
    }
}
