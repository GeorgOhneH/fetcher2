pub mod node;
pub mod opener;
pub mod root;

use std::convert::{TryFrom, TryInto};
use std::fmt::Display;
use std::marker::PhantomData;
use std::sync::Arc;

use druid::kurbo::{BezPath, Size};
use druid::piet::{LineCap, LineJoin, RenderContext, StrokeStyle};
use druid::widget::{Axis, ClipBox, Label, Scroll, Viewport};
use druid::{theme, Affine, Lens, LensExt, Rect, SingleUse, Vec2, WidgetExt};
use druid::{
    BoxConstraints, Data, Env, Event, EventCtx, LayoutCtx, LifeCycle, LifeCycleCtx, PaintCtx,
    Point, Selector, UpdateCtx, Widget, WidgetId, WidgetPod,
};

use crate::widgets::header::Header;
use crate::widgets::tree::node::{OpenerFactory, TreeNode};
use crate::widgets::tree::opener::make_wedge;
use crate::widgets::tree::root::{TreeNodeRoot, TreeNodeRootWidget};
use druid::im::Vector;
use druid::scroll_component::{ScrollComponent, ScrollbarsEnabled};
use druid_widget_nursery::selectors;
use std::ops::Mul;

pub enum SelectionMode {
    Single,
    Multiple,
}

enum SelectUpdateMode {
    Single,
    Add,
    Sub,
}

pub type NodeIndex = Vec<usize>;
pub type DataNodeIndex = Vector<usize>;

pub struct Tree<R, T, L, S, const N: usize>
where
    R: TreeNodeRoot<T>,
    T: TreeNode,
    L: Lens<T, bool>,
    S: Lens<R, Vector<DataNodeIndex>>,
{
    header: WidgetPod<R, ClipBox<R, Header<R, N>>>,
    root_node: WidgetPod<R, Scroll<R, TreeNodeRootWidget<R, T, L, N>>>,
    selected_lens: S,
    selection_mode: SelectionMode,

    on_activate_fn: Option<Box<dyn Fn(&mut EventCtx, &mut R, &Env, &NodeIndex)>>
}

/// Tree Implementation
impl<R, T, L, S, const N: usize> Tree<R, T, L, S, N>
where
    R: TreeNodeRoot<T>,
    T: TreeNode,
    L: Lens<T, bool> + Clone + 'static,
    S: Lens<R, Vector<DataNodeIndex>> + Clone + 'static,
{
    pub fn new(
        header_widgets: [impl Widget<R> + 'static; N],
        make_widgets: [Arc<dyn Fn() -> Box<dyn Widget<T>>>; N],
        expand_lens: L,
        selected_lens: S,
    ) -> Self {
        let el = expand_lens.clone();
        let make_opener: Arc<Box<OpenerFactory<T>>> =
            Arc::new(Box::new(move || Box::new(make_wedge(el.clone()))));
        let header = Header::columns(header_widgets).draggable(true);
        let constrains = header.constrains();

        Tree {
            header: WidgetPod::new(ClipBox::new(header).content_must_fill(true)),
            root_node: WidgetPod::new(
                TreeNodeRootWidget::new(make_widgets, make_opener, constrains, expand_lens)
                    .scroll(),
            ),
            selected_lens,
            selection_mode: SelectionMode::Single,
            on_activate_fn: None,
        }
    }

    pub fn on_activate(mut self, on_activate_fn: impl Fn(&mut EventCtx, &mut R, &Env, &NodeIndex) + 'static) -> Self {
        self.on_activate_fn = Some(Box::new(on_activate_fn));
        self
    }

    pub fn set_sizes(mut self, sizes: [f64; N]) -> Self {
        self.header.widget_mut().child_mut().sizes(sizes);
        let constrains = self.header.widget().child().constrains();
        self.root_node
            .widget_mut()
            .child_mut()
            .update_constrains(constrains);
        self
    }

    pub fn node_at(&self, p: Point) -> Option<NodeIndex> {
        let rect = self.root_node.layout_rect();
        if rect.contains(p) {
            self.root_node
                .widget()
                .child()
                .at(Point::new(p.x - rect.x0, p.y - rect.y0) + self.root_node.widget().offset())
        } else {
            None
        }
    }

    pub fn update_highlights(&mut self, p: Point) -> bool {
        let rect = self.root_node.layout_rect();
        let offset = self.root_node.widget().offset();
        if rect.contains(p) {
            self.root_node
                .widget_mut()
                .child_mut()
                .update_highlights(Point::new(p.x - rect.x0, p.y - rect.y0) + offset)
        } else {
            self.root_node.widget_mut().child_mut().remove_highlights()
        }
    }

    fn update_selection(&mut self, new_node: &NodeIndex, mode: SelectUpdateMode, data: &mut R) -> bool {
        let current_selected = self.root_node.widget().child().get_selected();
        match mode {
            SelectUpdateMode::Single => {
                if current_selected.len() == 1 && &current_selected[0] == new_node {
                    return false;
                }
                for selected_child_idx in &current_selected {
                    let node = self
                        .root_node
                        .widget_mut()
                        .child_mut()
                        .node_mut(selected_child_idx);
                    node.selected = false;
                }
                let node = self.root_node.widget_mut().child_mut().node_mut(&new_node);
                node.selected = true;
                self.selected_lens.with_mut(data, |selected| {
                    selected.clear();
                    selected.push_back(new_node.into())
                });
                true
            }
            SelectUpdateMode::Add => {
                if current_selected.contains(new_node) {
                    return false
                }
                let node = self.root_node.widget_mut().child_mut().node_mut(new_node);
                node.selected = true;
                self.selected_lens.with_mut(data, |selected| {
                    selected.push_back(new_node.into())
                });
                true
            }
            SelectUpdateMode::Sub => {
                if !current_selected.contains(new_node) {
                    return false;
                }
                let node = self.root_node.widget_mut().child_mut().node_mut(new_node);
                node.selected = false;
                self.selected_lens.with_mut(data, |selected| {
                    selected.retain(|idx| idx != &Vector::from(new_node))
                });
                true
            }
        }
    }

    fn header_offset(&self) -> f64 {
        self.root_node.widget().offset().x
    }
}
// Implement the Widget trait for Tree
impl<R, T, L, S, const N: usize> Widget<R> for Tree<R, T, L, S, N>
where
    R: TreeNodeRoot<T>,
    T: TreeNode,
    L: Lens<T, bool> + Clone + 'static,
    S: Lens<R, Vector<DataNodeIndex>> + Clone + 'static,
{
    fn event(&mut self, ctx: &mut EventCtx, event: &Event, data: &mut R, env: &Env) {
        self.root_node.event(ctx, event, data, env);

        let header_offset = self.header_offset();
        if self.header.widget_mut().viewport_origin().x != header_offset {
            ctx.request_layout()
        }
        self.header
            .widget_mut()
            .pan_to_on_axis(Axis::Horizontal, header_offset);
        self.header.event(ctx, event, data, env);

        if let Event::Wheel(mouse) = event {
            if self.update_highlights(mouse.pos) {
                ctx.request_paint()
            }
        }

        if ctx.is_handled() {
            return;
        }

        match event {
            Event::MouseDown(mouse_event) => {
                if let Some(idx) = self.node_at(mouse_event.pos) {
                    ctx.set_active(true);
                    ctx.set_handled();
                    if self.update_selection(&idx, SelectUpdateMode::Single, data) {
                        ctx.request_paint();
                    }
                    if mouse_event.count == 2 {
                        if let Some(activate_fn) = &self.on_activate_fn {
                            (activate_fn)(ctx, data, env, &idx)
                        }
                    }
                }
                return;
            }
            Event::MouseUp(_) => {
                if ctx.is_active() {
                    ctx.set_active(false);
                }
            }
            Event::MouseMove(mouse_event) => {
                if self.update_highlights(mouse_event.pos) {
                    ctx.request_paint();
                }
                if ctx.is_active() {
                    if let Some(idx) = self.node_at(mouse_event.pos) {
                        let mode = match self.selection_mode {
                            SelectionMode::Single => SelectUpdateMode::Single,
                            SelectionMode::Multiple => SelectUpdateMode::Add,
                        };
                        if self.update_selection(&idx, mode, data) {
                            ctx.request_paint();
                        }
                    }
                }
            }
            _ => (),
        }
    }

    fn lifecycle(&mut self, ctx: &mut LifeCycleCtx, event: &LifeCycle, data: &R, env: &Env) {
        if let LifeCycle::HotChanged(false) = event {
            self.root_node.widget_mut().child_mut().remove_highlights();
        }
        self.root_node.lifecycle(ctx, event, data, env);
        self.header.lifecycle(ctx, event, data, env);
    }

    fn update(&mut self, ctx: &mut UpdateCtx, _old_data: &R, data: &R, env: &Env) {
        //TODO lens update select
        self.header.update(ctx, data, env);
        self.root_node.update(ctx, data, env);
    }

    fn layout(&mut self, ctx: &mut LayoutCtx, bc: &BoxConstraints, data: &R, env: &Env) -> Size {
        let header_bc = BoxConstraints::new(Size::new(bc.min().width, 0.), bc.max());
        let header_size = self.header.layout(ctx, &header_bc, data, env);

        let constrains = self.header.widget().child().constrains();
        self.root_node
            .widget_mut()
            .child_mut()
            .update_constrains(constrains);

        let node_bc = BoxConstraints::new(
            Size::new(
                bc.min().width,
                (bc.min().height - header_size.height).max(0.),
            ),
            Size::new(
                bc.max().width,
                (bc.max().height - header_size.height).max(0.),
            ),
        );
        let root_size = self.root_node.layout(ctx, &node_bc, data, env);

        self.header.set_origin(ctx, data, env, Point::ORIGIN);
        self.root_node
            .set_origin(ctx, data, env, Point::new(0., header_size.height));
        // TODO: ctx.set_paint_insets...
        let my_size = Size::new(header_size.width, header_size.height + root_size.height);

        bc.constrain(my_size)
    }

    fn paint(&mut self, ctx: &mut PaintCtx, data: &R, env: &Env) {
        let rect = ctx.size().to_rect();
        ctx.fill(rect, &env.get(theme::BACKGROUND_LIGHT));

        self.root_node.paint(ctx, data, env);
        self.header.paint(ctx, data, env);
    }
}