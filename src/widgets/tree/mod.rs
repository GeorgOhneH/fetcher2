pub mod highlight_box;
pub mod node;
pub mod opener;
pub mod root;

use std::convert::{TryFrom, TryInto};
use std::fmt::Display;
use std::marker::PhantomData;
use std::sync::Arc;

use druid::kurbo::{BezPath, Size};
use druid::piet::{LineCap, LineJoin, RenderContext, StrokeStyle};
use druid::widget::Label;
use druid::{theme, Lens, LensExt, Rect, SingleUse};
use druid::{
    BoxConstraints, Data, Env, Event, EventCtx, LayoutCtx, LifeCycle, LifeCycleCtx, PaintCtx,
    Point, Selector, UpdateCtx, Widget, WidgetId, WidgetPod,
};

use crate::template::NodeIndex;
use crate::widgets::header::{Header, HEADER_SIZE_CHANGED};
use crate::widgets::tree::node::{OpenerFactory, TreeNode};
use crate::widgets::tree::opener::make_wedge;
use crate::widgets::tree::root::{TreeNodeRoot, TreeNodeRootWidget};
use druid_widget_nursery::selectors;

pub enum SelectionMode {
    Nothing,
    Single,
    Multiple,
}

pub struct Tree<R, T, L, const N: usize>
where
    R: TreeNodeRoot<T>,
    T: TreeNode,
    L: Lens<T, bool>,
{
    header: WidgetPod<R, Header<R, N>>,
    /// The root node of this tree
    root_node: WidgetPod<R, TreeNodeRootWidget<R, T, L, N>>,
    selected: Option<NodeIndex>,
    selection_mode: SelectionMode,
}

/// Tree Implementation
impl<R: TreeNodeRoot<T>, T: TreeNode, L: Lens<T, bool> + Clone + 'static, const N: usize>
    Tree<R, T, L, N>
{
    pub fn new(
        header_widgets: [impl Widget<R> + 'static; N],
        make_widgets: [Arc<dyn Fn() -> Box<dyn Widget<T>>>; N],
        expand_lens: L,
    ) -> Self {
        let el = expand_lens.clone();
        let make_opener: Arc<Box<OpenerFactory<T>>> =
            Arc::new(Box::new(move || Box::new(make_wedge(el.clone()))));
        let header = Header::columns(header_widgets).draggable(true);
        let sizes = header.widget_pos();
        Tree {
            header: WidgetPod::new(header),
            root_node: WidgetPod::new(TreeNodeRootWidget::new(
                make_widgets,
                make_opener,
                sizes,
                expand_lens,
            )),
            selected: None,
            selection_mode: SelectionMode::Single,
        }
    }

    // pub fn with_opener<W: Widget<T> + 'static>(
    //     mut self,
    //     closure: impl Fn() -> W + 'static,
    // ) -> Self {
    //     self.root_node.widget_mut().make_opener = Arc::new(Box::new(move || Box::new(closure())));
    //     self.root_node.widget_mut().opener = WidgetPod::new(Opener {
    //         widget: WidgetPod::new(self.root_node.widget_mut().make_opener.clone()()),
    //     });
    //     self
    // }
}
impl<R: TreeNodeRoot<T>, T: TreeNode, L: Lens<T, bool> + Clone + 'static, const N: usize>
    Tree<R, T, L, N>
{
    pub fn node_at(&self, p: Point) -> Option<NodeIndex> {
        let rect = self.root_node.layout_rect();
        if rect.contains(p) {
            self.root_node
                .widget()
                .at(Point::new(p.x - rect.x0, p.y - rect.y0))
        } else {
            None
        }
    }
}
// Implement the Widget trait for Tree
impl<R: TreeNodeRoot<T>, T: TreeNode, L: Lens<T, bool> + Clone + 'static, const N: usize> Widget<R>
    for Tree<R, T, L, N>
{
    fn event(&mut self, ctx: &mut EventCtx, event: &Event, data: &mut R, env: &Env) {
        self.header.event(ctx, event, data, env);
        if ctx.is_handled() {
            return;
        }
        self.root_node.event(ctx, event, data, env);
        if ctx.is_handled() {
            return;
        }

        match event {
            Event::Notification(notif) if notif.is(HEADER_SIZE_CHANGED) => {
                ctx.set_handled();
                let sizes = self.header.widget().widget_pos();
                self.root_node.widget_mut().update_sizes(sizes);
                ctx.request_layout();
                return;
            }
            Event::MouseDown(mouse_event) => {
                if let Some(idx) = self.node_at(mouse_event.pos) {
                    ctx.set_active(true);
                    for selected_child_idx in self.root_node.widget().get_selected() {
                        let node = self.root_node.widget_mut().node_mut(&selected_child_idx);
                        node.highlight_box.widget_mut().selected = false;
                    }
                    let node = self.root_node.widget_mut().node_mut(&idx);
                    node.highlight_box.widget_mut().selected = true;
                    ctx.request_paint();
                }
                return;
            }
            Event::MouseUp(_) => {
                if ctx.is_active() {
                    ctx.set_active(false);
                }
            }
            Event::MouseMove(mouse_event) => {
                if ctx.is_active() {
                    if let Some(idx) = self.node_at(mouse_event.pos) {
                        if matches!(self.selection_mode, SelectionMode::Single) {
                            for selected_child_idx in self.root_node.widget().get_selected() {
                                let node = self.root_node.widget_mut().node_mut(&selected_child_idx);
                                node.highlight_box.widget_mut().selected = false;
                            }
                        }
                        let node = self.root_node.widget_mut().node_mut(&idx);
                        node.highlight_box.widget_mut().selected = true;
                        ctx.request_paint();
                    }
                }
            }
            _ => (),
        }
    }

    fn lifecycle(&mut self, ctx: &mut LifeCycleCtx, event: &LifeCycle, data: &R, env: &Env) {
        self.header.lifecycle(ctx, event, data, env);
        self.root_node.lifecycle(ctx, event, data, env);
    }

    fn update(&mut self, ctx: &mut UpdateCtx, _old_data: &R, data: &R, env: &Env) {
        self.header.update(ctx, data, env);
        self.root_node.update(ctx, data, env);
    }

    fn layout(&mut self, ctx: &mut LayoutCtx, bc: &BoxConstraints, data: &R, env: &Env) -> Size {
        let header_size = self.header.layout(ctx, bc, data, env);
        let node_bc = BoxConstraints::new(
            Size::new(header_size.width, 0.),
            Size::new(header_size.width, f64::INFINITY),
        );
        let content_size = self.root_node.layout(ctx, &node_bc, data, env);
        self.header.set_origin(ctx, data, env, Point::ORIGIN);
        self.root_node
            .set_origin(ctx, data, env, Point::new(0., header_size.height));
        // TODO: ctx.set_paint_insets...
        let my_size = Size::new(header_size.width, header_size.height + content_size.height);
        bc.constrain(my_size)
    }

    fn paint(&mut self, ctx: &mut PaintCtx, data: &R, env: &Env) {
        let rect = ctx.size().to_rect();
        ctx.fill(rect, &env.get(theme::BACKGROUND_LIGHT));
        self.header.paint(ctx, data, env);
        self.root_node.paint(ctx, data, env);
    }
}
