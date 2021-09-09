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
use druid::widget::{ClipBox, Label, Scroll, Viewport};
use druid::{theme, Affine, Lens, LensExt, Rect, SingleUse, Vec2, WidgetExt};
use druid::{
    BoxConstraints, Data, Env, Event, EventCtx, LayoutCtx, LifeCycle, LifeCycleCtx, PaintCtx,
    Point, Selector, UpdateCtx, Widget, WidgetId, WidgetPod,
};

use crate::template::NodeIndex;
use crate::widgets::header::Header;
use crate::widgets::tree::node::{OpenerFactory, TreeNode};
use crate::widgets::tree::opener::make_wedge;
use crate::widgets::tree::root::{TreeNodeRoot, TreeNodeRootWidget};
use druid::scroll_component::{ScrollComponent, ScrollbarsEnabled};
use druid_widget_nursery::selectors;
use std::ops::Mul;

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
    root_node: WidgetPod<R, Scroll<R, TreeNodeRootWidget<R, T, L, N>>>,
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
        let constrains = header.constrains();

        Tree {
            header: WidgetPod::new(header),
            root_node: WidgetPod::new(
                TreeNodeRootWidget::new(make_widgets, make_opener, constrains, expand_lens)
                    .scroll(),
            ),
            selected: None,
            selection_mode: SelectionMode::Single,
        }
    }

    pub fn set_sizes(mut self, sizes: [f64; N]) -> Self {
        self.header.widget_mut().sizes(sizes);
        let constrains = self.header.widget().constrains();
        self.root_node
            .widget_mut()
            .child_mut()
            .update_constrains(constrains);
        self
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
                .child()
                .at(Point::new(p.x - rect.x0, p.y - rect.y0) + self.root_node.widget().offset())
        } else {
            None
        }
    }

    fn header_offset(&self) -> Vec2 {
        let mut header_offset = self.root_node.widget().offset();
        header_offset.y = 0.;
        header_offset
    }
}
// Implement the Widget trait for Tree
impl<R: TreeNodeRoot<T>, T: TreeNode, L: Lens<T, bool> + Clone + 'static, const N: usize> Widget<R>
    for Tree<R, T, L, N>
{
    fn event(&mut self, ctx: &mut EventCtx, event: &Event, data: &mut R, env: &Env) {
        self.root_node.event(ctx, event, data, env);
        self.header.set_viewport_offset(self.header_offset());

        if !ctx.is_handled() {
            let viewport = self.header.layout_rect();
            let header_force_event = self.header.is_hot() || self.header.has_active();
            if let Some(child_event) =
                event.transform_scroll(self.header_offset(), viewport, header_force_event)
            {
                self.header.event(ctx, &child_event, data, env);
            }
        }

        match event {
            Event::MouseDown(mouse_event) => {
                if let Some(idx) = self.node_at(mouse_event.pos) {
                    ctx.set_active(true);
                    for selected_child_idx in self.root_node.widget().child().get_selected() {
                        let node = self.root_node.widget_mut().child_mut().node_mut(&selected_child_idx);
                        node.highlight_box.widget_mut().selected = false;
                    }
                    let node = self.root_node.widget_mut().child_mut().node_mut(&idx);
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
                            for selected_child_idx in self.root_node.widget().child().get_selected() {
                                let node =
                                    self.root_node.widget_mut().child_mut().node_mut(&selected_child_idx);
                                node.highlight_box.widget_mut().selected = false;
                            }
                        }
                        let node = self.root_node.widget_mut().child_mut().node_mut(&idx);
                        node.highlight_box.widget_mut().selected = true;
                        ctx.request_paint();
                    }
                }
            }
            _ => (),
        }
    }

    fn lifecycle(&mut self, ctx: &mut LifeCycleCtx, event: &LifeCycle, data: &R, env: &Env) {
        self.root_node.lifecycle(ctx, event, data, env);
        self.header.lifecycle(ctx, event, data, env);
    }

    fn update(&mut self, ctx: &mut UpdateCtx, _old_data: &R, data: &R, env: &Env) {
        self.header.update(ctx, data, env);
        self.root_node.update(ctx, data, env);
    }

    fn layout(&mut self, ctx: &mut LayoutCtx, bc: &BoxConstraints, data: &R, env: &Env) -> Size {
        let header_bc = BoxConstraints::new(
            Size::new(bc.min().width, 0.),
            Size::new(f64::INFINITY, f64::INFINITY),
        );
        let header_size = self.header.layout(ctx, &header_bc, data, env);

        let constrains = self.header.widget().constrains();
        self.root_node.widget_mut().child_mut().update_constrains(constrains);

        let node_bc = BoxConstraints::new(
            Size::new(bc.min().width, (bc.min().height-header_size.height).max(0.)),
            Size::new(bc.max().width, (bc.max().height-header_size.height).max(0.)),
        );
        let root_size = self.root_node.layout(ctx, &node_bc, data, env);

        self.header.set_origin(ctx, data, env, Point::ORIGIN);
        self.root_node
            .set_origin(ctx, data, env, Point::new(0., header_size.height));
        // TODO: ctx.set_paint_insets...
        let my_size = Size::new(header_size.width, header_size.height + root_size.height);

        self.header.set_viewport_offset(self.header_offset());

        bc.constrain(my_size)
    }

    fn paint(&mut self, ctx: &mut PaintCtx, data: &R, env: &Env) {
        let rect = ctx.size().to_rect();
        ctx.fill(rect, &env.get(theme::BACKGROUND_LIGHT));

        self.root_node.paint(ctx, data, env);

        let header_offset = self.header_offset();
        dbg!(header_offset);
        ctx.with_save(|ctx| {
            ctx.clip(self.header.layout_rect());
            ctx.transform(Affine::translate(-header_offset));

            let mut visible = ctx.region().clone();
            visible += header_offset;
            dbg!(&visible);
            ctx.with_child_ctx(visible, |ctx| self.header.paint(ctx, data, env));
        });

    }
}
