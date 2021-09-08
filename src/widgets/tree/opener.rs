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
use crate::widgets::header::{Header};
use druid_widget_nursery::selectors;
use crate::widgets::tree::root::TreeNodeRoot;
use crate::widgets::tree::node::{TreeNode, TREE_ACTIVATE_NODE};

/// A tree widget for a collection of items organized in a hierarchical way.

// Wrapper widget that reacts to clicks by sending a TREE_ACTIVATE_NODE command to
// its inner user-defined widget.
// TODO: Try use a Controller instead of a plain widget.
pub struct Opener<T>
    where
        T: TreeNode,
{
    widget: WidgetPod<T, Box<dyn Widget<T>>>,
}


impl<T: TreeNode> Opener<T> {
    pub fn new(widget: Box<dyn Widget<T>>) -> Self {
        Self {
            widget: WidgetPod::new(widget)
        }
    }
}

/// Implementing Widget for the Opener.
impl<T: TreeNode> Widget<T> for Opener<T>
    where
        T: Data,
{
    fn event(&mut self, ctx: &mut EventCtx, event: &Event, data: &mut T, _env: &Env) {
        match event {
            Event::MouseDown(_) => {
                ctx.set_active(true);
                ctx.request_paint();
                ctx.set_handled()
            }
            Event::MouseUp(_) => {
                if ctx.is_active() {
                    ctx.set_active(false);
                    if ctx.is_hot() {
                        ctx.submit_command(TREE_ACTIVATE_NODE.to(self.widget.id()));
                    }
                    ctx.request_paint();
                }
            }
            _ => (),
        }
        self.widget.event(ctx, event, data, _env);
    }

    fn lifecycle(&mut self, ctx: &mut LifeCycleCtx, event: &LifeCycle, data: &T, env: &Env) {
        self.widget.lifecycle(ctx, event, data, env);
    }

    fn update(&mut self, ctx: &mut UpdateCtx, _old_data: &T, data: &T, env: &Env) {
        self.widget.update(ctx, data, env)
    }

    fn layout(&mut self, ctx: &mut LayoutCtx, bc: &BoxConstraints, data: &T, env: &Env) -> Size {
        let size = self.widget.layout(ctx, bc, data, env);
        self.widget.set_origin(ctx, data, env, Point::ORIGIN);
        bc.constrain(size)
    }

    fn paint(&mut self, ctx: &mut PaintCtx, data: &T, env: &Env) {
        self.widget.paint(ctx, data, env)
    }
}

// The default opener if none is passed to the Tree builder.
pub struct Wedge<T, L>
    where
        T: TreeNode,
        L: Lens<T, bool>,
{
    expand_lens: L,
    phantom: PhantomData<T>,
}

impl<T: TreeNode, L: Lens<T, bool>> Widget<T> for Wedge<T, L> {
    fn event(&mut self, ctx: &mut EventCtx, event: &Event, data: &mut T, _env: &Env) {
        match event {
            Event::Command(cmd) if cmd.is(TREE_ACTIVATE_NODE) => {
                self.expand_lens.put(data, !self.expand_lens.get(data));
                ctx.set_handled();
            }
            _ => (),
        }
    }

    fn lifecycle(&mut self, _ctx: &mut LifeCycleCtx, _event: &LifeCycle, _data: &T, _env: &Env) {}

    fn update(&mut self, _ctx: &mut UpdateCtx, _old_data: &T, _data: &T, _env: &Env) {}

    fn layout(&mut self, _ctx: &mut LayoutCtx, bc: &BoxConstraints, _data: &T, env: &Env) -> Size {
        let size = env.get(theme::BASIC_WIDGET_HEIGHT);
        bc.constrain(Size::new(size, size))
    }

    fn paint(&mut self, ctx: &mut PaintCtx, data: &T, env: &Env) {
        if !data.is_branch() {
            return;
        }
        let stroke_color = if ctx.is_hot() {
            env.get(theme::FOREGROUND_LIGHT)
        } else {
            env.get(theme::FOREGROUND_DARK)
        };

        // Paint the opener
        let mut path = BezPath::new();
        if self.expand_lens.get(data) {
            // expanded: 'V' shape
            path.move_to((5.0, 7.0));
            path.line_to((9.0, 13.0));
            path.line_to((13.0, 7.0));
        } else {
            // collapsed: '>' shape
            path.move_to((7.0, 5.0));
            path.line_to((13.0, 9.0));
            path.line_to((7.0, 13.0));
        }
        let style = StrokeStyle::new()
            .line_cap(LineCap::Round)
            .line_join(LineJoin::Round);

        ctx.stroke_styled(path, &stroke_color, 2.5, &style);
    }
}

pub fn make_wedge<T: TreeNode, L: Lens<T, bool>>(expand_lens: L) -> Wedge<T, L> {
    Wedge {
        phantom: PhantomData,
        expand_lens,
    }
}
