use std::convert::TryFrom;
use std::fmt::Display;
use std::sync::Arc;

use druid::kurbo::{BezPath, Size};
use druid::piet::{LineCap, LineJoin, RenderContext, StrokeStyle};
use druid::widget::{Controller, Label, RawLabel};
use druid::{theme, Rect, Selector, SingleUse, WidgetExt, WidgetId, LinearGradient, UnitPoint};
use druid::{
    BoxConstraints, Data, Env, Event, EventCtx, LayoutCtx, LifeCycle, LifeCycleCtx, PaintCtx,
    Point, UpdateCtx, Widget, WidgetPod,
};

use crate::template::communication::NODE_EVENT;
use crate::template::node_type::site::SiteState;
use crate::template::node_type::site::{
    DownloadEvent, LoginEvent, RunEvent, SiteEvent, UrlFetchEvent,
};
use crate::template::node_type::NodeTypeData;
use crate::template::nodes::node::{NodeEvent, PathEvent};
use crate::template::nodes::node_data::NodeData;
use crate::template::widget::TemplateWidget;
use crate::template::MetaData;
use crate::TError;
use druid::im::Vector;
use druid_widget_nursery::{selectors, Wedge};
use std::path::PathBuf;

selectors! {
    TREE_OPEN_PARENT,
}

pub const SELECT: Selector<SingleUse<Vec<usize>>> =
    Selector::new("fetcher2.template_widget.select");

pub struct NodeWidget {
    pub index: usize,

    wedge: WidgetPod<bool, Wedge>,

    widgets: Vec<WidgetPod<NodeData, Box<dyn Widget<NodeData>>>>,

    expanded: bool,

    children: Vec<WidgetPod<NodeData, Self>>,

    section_bc: Option<Vec<(f64, f64)>>,
    section_num: usize,

    selected: bool,
}

impl NodeWidget {
    /// Create a TreeNodeWidget from a TreeNode.
    pub fn new(expanded: bool, id: WidgetId) -> Self {
        NodeWidget {
            index: 0,
            wedge: WidgetPod::new(Wedge::new()),
            widgets: vec![WidgetPod::new(Box::new(NodeData::widget(0).with_id(id)))],
            expanded,
            children: Vec::new(),
            section_bc: None,
            section_num: 0,
            selected: false,
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

    pub fn set_section_bc(&mut self, section_bc: Vec<(f64, f64)>) {
        self.section_bc = Some(section_bc)
    }

    pub fn set_section_num(&mut self, section_num: usize) {
        self.section_num = section_num;
        for i in 1..section_num {
            self.widgets.push(WidgetPod::new(NodeData::widget(i)));
        }
        for widget in self.children.iter_mut() {
            widget.widget_mut().set_section_num(section_num)
        }
    }

    pub fn updated_selection(&mut self, selection: &[usize]) {
        if selection.is_empty() {
            self.selected = true;
            for widget in self.children.iter_mut() {
                widget.widget_mut().unselect()
            }
            return;
        }
        self.selected = false;
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

    pub fn unselect(&mut self) {
        self.selected = false;
        for widget in self.children.iter_mut() {
            widget.widget_mut().unselect()
        }
    }
}

impl Widget<NodeData> for NodeWidget {
    fn event(&mut self, ctx: &mut EventCtx, event: &Event, data: &mut NodeData, env: &Env) {
        // eprintln!("{:?}", event);
        match event {
            Event::MouseDown(_) => {
                ctx.set_active(true);
                ctx.request_paint();
            }
            Event::MouseUp(_) => {
                if ctx.is_active() {
                    ctx.set_active(false);
                    if ctx.is_hot() {
                        ctx.submit_notification(SELECT.with(SingleUse::new(vec![self.index])));
                    }
                    ctx.request_paint();
                }
            }
            Event::Notification(notif) => {
                if notif.is(TREE_OPEN_PARENT) {
                    ctx.set_handled();
                    self.expanded = true;
                    ctx.children_changed();
                } else if let Some(select) = notif.get(SELECT) {
                    ctx.set_handled();
                    let mut current_index = select.take().unwrap();
                    current_index.push(self.index);
                    ctx.submit_notification(SELECT.with(SingleUse::new(current_index)));
                }
                return;
            },
            _ => (),
        }

        for widget in self.widgets.iter_mut() {
            widget.event(ctx, event, data, env);
        }

        for (child_widget_node, child_data) in
            self.children.iter_mut().zip(data.children.iter_mut())
        {
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
        for widget in self.widgets.iter_mut() {
            widget.lifecycle(ctx, event, data, env);
        }
        for (child_widget_node, child_data) in self.children.iter_mut().zip(data.children.iter()) {
            child_widget_node.lifecycle(ctx, event, child_data, env);
        }
        if let LifeCycle::HotChanged(_) = event {
            ctx.request_paint();
        }
    }

    fn update(&mut self, ctx: &mut UpdateCtx, old_data: &NodeData, data: &NodeData, env: &Env) {
        if !old_data.same(data) {
            // eprintln!("not same");
            // eprintln!("{:?}", old_data);
            // eprintln!("{:?}", data);
            self.wedge.update(ctx, &self.expanded, env);
            for widget in self.widgets.iter_mut() {
                widget.update(ctx, data, env);
            }
            for (child_widget_node, child_data) in
                self.children.iter_mut().zip(data.children.iter())
            {
                child_widget_node.update(ctx, child_data, env);
            }
            ctx.request_layout();
            ctx.children_changed();
        }
    }

    fn layout(
        &mut self,
        ctx: &mut LayoutCtx,
        bc: &BoxConstraints,
        data: &NodeData,
        env: &Env,
    ) -> Size {
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
        let mut max_widget_height = 0.;
        for (i, ((point, width_constraint), widget)) in self
            .section_bc
            .as_ref()
            .unwrap()
            .iter()
            .zip(self.widgets.iter_mut())
            .enumerate()
        {
            let widget_size = widget.layout(
                ctx,
                &BoxConstraints::new(
                    Size::new(min_width, 0.),
                    Size::new(*width_constraint, basic_size * 1.2),
                ),
                data,
                env,
            );
            max_widget_height = widget_size.height.max(max_widget_height);
            if i == 0 {
                widget.set_origin(ctx, data, env, Point::new(basic_size + point, 0.0));
            } else {
                widget.set_origin(ctx, data, env, Point::new(*point, 0.0));
            }
        }

        // This is the computed size of this node. We start with the size of the widget,
        // and will increase for each child node.
        let mut size = Size::new(max_width, max_widget_height);

        // Below, the children nodes, but only if expanded
        if self.expanded && max_width > indent {
            if min_width > indent {
                min_width -= min_width;
            } else {
                min_width = 0.0;
            }
            max_width -= indent;

            let child_section_bc: Vec<_> = self
                .section_bc
                .as_ref()
                .unwrap()
                .iter()
                .enumerate()
                .map(|(i, (x, width))| {
                    if i == 0 {
                        (*x, *width)
                    } else {
                        (x - indent, *width)
                    }
                })
                .collect();

            for (child_widget_node, child_data) in
                self.children.iter_mut().zip(data.children.iter())
            {
                // In case we have lazily instanciated children nodes,
                // we may skip some indices. This catches up the correct height.

                // Layout and position a child node
                let child_bc = BoxConstraints::new(
                    Size::new(min_width, 0.0),
                    Size::new(max_width, f64::INFINITY),
                );
                child_widget_node
                    .widget_mut()
                    .set_section_bc(child_section_bc.clone());
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
        let mut highlight_rect = self.wedge.layout_rect();
        let mut any_child_hot = false;
        for widget in self.widgets.iter_mut() {
            let clip_rect = widget.layout_rect();
            highlight_rect = highlight_rect.union(clip_rect);
            any_child_hot = widget.is_hot() || any_child_hot;
            let background_color = env.get(theme::BACKGROUND_LIGHT);
            ctx.fill(clip_rect, &background_color);
        }

        if self.selected {
            let background_gradient = LinearGradient::new(
                UnitPoint::TOP,
                UnitPoint::BOTTOM,
                (env.get(theme::PRIMARY_LIGHT), env.get(theme::PRIMARY_DARK)),
            );
            ctx.fill(highlight_rect, &background_gradient);
        } else if ctx.is_active() {
            let background_gradient = LinearGradient::new(
                UnitPoint::TOP,
                UnitPoint::BOTTOM,
                (
                    env.get(theme::BACKGROUND_LIGHT),
                    env.get(theme::BACKGROUND_DARK),
                ),
            );
            ctx.fill(highlight_rect, &background_gradient);
        }


        if ctx.is_hot() && !any_child_hot {
            ctx.stroke(highlight_rect, &env.get(theme::BORDER_LIGHT), 1.);
        }


        for widget in self.widgets.iter_mut() {
            widget.paint(ctx, data, env);
        }

        if self.expanded {
            for (child_widget_node, child_data) in
                self.children.iter_mut().zip(data.children.iter())
            {
                child_widget_node.paint(ctx, child_data, env);
            }
        }
    }
}
