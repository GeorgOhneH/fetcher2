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
use crate::template::MetaData;
use std::path::PathBuf;
use druid::im::Vector;
use crate::template::communication::PATH_UPDATED;

selectors! {
    TREE_OPEN_PARENT,
}


#[derive(Data, Clone, Debug)]
pub struct NodeData {
    pub ty: NodeTypeData,
    pub meta_data: MetaData,
    pub children: Vector<NodeData>,

    #[data(same_fn = "PartialEq::eq")]
    pub cached_path: Option<PathBuf>,
}

impl NodeData {
    pub fn widget() -> impl Widget<Self> {
        Label::dynamic(|data: &NodeData, _env| format!("{:?}", data.ty)).controller(TemplateUpdate)
    }
}

struct TemplateUpdate;

impl<W: Widget<NodeData>> Controller<NodeData, W> for TemplateUpdate {
    fn event(&mut self, child: &mut W, ctx: &mut EventCtx, event: &Event, data: &mut NodeData, env: &Env) {
        if let Event::Command(cmd) = event {
            if let Some(path) = cmd.get(PATH_UPDATED) {
                data.cached_path = Some(path.take().unwrap());
                return;
            }
        }
        child.event(ctx, event, data, env)
    }
}

pub struct NodeWidget
{
    pub index: usize,

    wedge: WidgetPod<bool, Wedge>,

    widget: WidgetPod<NodeData, Box<dyn Widget<NodeData>>>,

    expanded: bool,

    children: Vec<WidgetPod<NodeData, Self>>,
}

impl NodeWidget {
    /// Create a TreeNodeWidget from a TreeNode.
    pub fn new(expanded: bool, id: WidgetId) -> Self {
        NodeWidget {
            index: 0,
            wedge: WidgetPod::new(Wedge::new()),
            widget: WidgetPod::new(Box::new(NodeData::widget().with_id(id))),
            expanded,
            children: Vec::new(),
        }
    }

    pub fn add_child(&mut self, mut child: Self) {
        child.index = self.children.len();
        self.children.push(WidgetPod::new(child));
    }

    pub fn add_children(&mut self, children: Vec<NodeWidget>) {
        for child in children.into_iter() {
            self.add_child(child)
        }
    }

}

impl Widget<NodeData> for NodeWidget
{
    fn event(&mut self, ctx: &mut EventCtx, event: &Event, data: &mut NodeData, env: &Env) {
        // eprintln!("{:?}", event);
        if let Event::Notification(notif) = event {
            if notif.is(TREE_OPEN_PARENT) {
                ctx.set_handled();
                self.expanded = true;
                ctx.children_changed();
            }
            return;
        }

        self.widget.event(ctx, event, data, env);

        for (child_widget_node, child_data) in self.children.iter_mut().zip(data.children.iter_mut()) {
            child_widget_node.event(ctx, event, child_data, env);
        }

        // Propagate the event to the wedge
        let mut wegde_expanded = self.expanded;
        self.wedge.event(ctx, event, &mut wegde_expanded, env);

        // Handle possible creation of new children nodes
        if let Event::MouseUp(_) = event {
            if wegde_expanded != self.expanded {
                // The wedge widget has decided to change the expanded/collapsed state of the node,
                // handle it by expanding/collapsing children nodes as required.
                ctx.request_layout();
                self.expanded = wegde_expanded;
            }
        }
    }

    fn lifecycle(&mut self, ctx: &mut LifeCycleCtx, event: &LifeCycle, data: &NodeData, env: &Env) {
        // eprintln!("{:?}", event);
        self.wedge.lifecycle(ctx, event, &self.expanded, env);
        self.widget.lifecycle(ctx, event, data, env);
        for (child_widget_node, child_data) in self.children.iter_mut().zip(data.children.iter()) {
            child_widget_node.lifecycle(ctx, event, child_data, env);
        }
    }

    fn update(&mut self, ctx: &mut UpdateCtx, old_data: &NodeData, data: &NodeData, env: &Env) {
        if !old_data.same(data) {
            // eprintln!("not same");
            // eprintln!("{:?}", old_data);
            // eprintln!("{:?}", data);
            self.wedge.update(ctx, &self.expanded, env);
            self.widget.update(ctx, data, env);
            for (child_widget_node, child_data) in self.children.iter_mut().zip(data.children.iter()) {
                child_widget_node.update(ctx, child_data, env);
            }
            ctx.request_layout();
            ctx.children_changed();
        }
    }

    // TODO: the height calculation seems to ignore the inner widget (at least on X11). issue #61
    fn layout(&mut self, ctx: &mut LayoutCtx, bc: &BoxConstraints, data: &NodeData, env: &Env) -> Size {
        let basic_size = env.get(theme::BASIC_WIDGET_HEIGHT);
        let indent = env.get(theme::BASIC_WIDGET_HEIGHT); // For a lack of a better definition
        let mut min_width = bc.min().width;
        let mut max_width = bc.max().width;

        // Top left, the wedge
        self.wedge.layout(
            ctx,
            &BoxConstraints::tight(Size::new(basic_size, basic_size)),
            &self.expanded,
            env,
        );
        self.wedge
            .set_origin(ctx, &self.expanded, env, Point::ORIGIN);

        // Immediately on the right, the node widget
        let widget_size = self.widget.layout(
            ctx,
            &BoxConstraints::new(
                Size::new(min_width, basic_size),
                Size::new(max_width, basic_size),
            ),
            data,
            env,
        );
        self.widget
            .set_origin(ctx, data, env, Point::new(basic_size, 0.0));

        // This is the computed size of this node. We start with the size of the widget,
        // and will increase for each child node.
        let mut size = Size::new(indent + widget_size.width, basic_size);

        // Below, the children nodes, but only if expanded
        if self.expanded && max_width > indent {
            if min_width > indent {
                min_width -= min_width;
            } else {
                min_width = 0.0;
            }
            max_width -= indent;

            for (child_widget_node, child_data) in self.children.iter_mut().zip(data.children.iter()) {
                // In case we have lazily instanciated children nodes,
                // we may skip some indices. This catches up the correct height.

                // Layout and position a child node
                let child_bc = BoxConstraints::new(
                    Size::new(min_width, 0.0),
                    Size::new(max_width, f64::INFINITY),
                );
                let child_size = child_widget_node.layout(ctx, &child_bc, child_data, env);
                let child_pos = Point::new(indent, size.height); // We position the child at the current height
                child_widget_node.set_origin(ctx, child_data, env, child_pos);
                size.height += child_size.height; // Increment the height of this node by the height of this child node
                if indent + child_size.width > size.width {
                    size.width = indent + child_size.width;
                }
            }
        }
        bc.constrain(size)
    }

    fn paint(&mut self, ctx: &mut PaintCtx, data: &NodeData, env: &Env) {
        if data.children.len() > 0 {
            // we paint the wedge only if there are children to expand
            self.wedge.paint(ctx, &self.expanded, env);
        }
        self.widget.paint(ctx, data, env);
        if self.expanded {
            for (child_widget_node, child_data) in self.children.iter_mut().zip(data.children.iter()) {
                child_widget_node.paint(ctx, child_data, env);
            }
        }
    }
}
