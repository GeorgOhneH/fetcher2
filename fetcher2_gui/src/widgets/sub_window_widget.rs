use druid::{Point, Widget, WidgetPod};
use druid::widget::prelude::*;

use crate::data::win::{SubWindowInfo, WindowState};

pub struct SubWindow<T> {
    child: WidgetPod<T, Box<dyn Widget<T>>>,
}

impl<T> SubWindow<T> {
    pub fn new(child: impl Widget<T> + 'static) -> Self {
        Self {
            child: WidgetPod::new(Box::new(child)),
        }
    }
}

impl<T: Data> Widget<SubWindowInfo<T>> for SubWindow<T> {
    fn event(&mut self, ctx: &mut EventCtx, event: &Event, data: &mut SubWindowInfo<T>, env: &Env) {
        if let Event::WindowCloseRequested = event {
            data.win_state = Some(WindowState::from_win(ctx.window()));
        }
        self.child.event(ctx, event, &mut data.data, env)
    }

    fn lifecycle(
        &mut self,
        ctx: &mut LifeCycleCtx,
        event: &LifeCycle,
        data: &SubWindowInfo<T>,
        env: &Env,
    ) {
        self.child.lifecycle(ctx, event, &data.data, env)
    }

    fn update(
        &mut self,
        ctx: &mut UpdateCtx,
        _old_data: &SubWindowInfo<T>,
        data: &SubWindowInfo<T>,
        env: &Env,
    ) {
        self.child.update(ctx, &data.data, env)
    }

    fn layout(
        &mut self,
        ctx: &mut LayoutCtx,
        bc: &BoxConstraints,
        data: &SubWindowInfo<T>,
        env: &Env,
    ) -> Size {
        let size = self.child.layout(ctx, bc, &data.data, env);
        self.child.set_origin(ctx, &data.data, env, Point::ORIGIN);
        size
    }

    fn paint(&mut self, ctx: &mut PaintCtx, data: &SubWindowInfo<T>, env: &Env) {
        self.child.paint(ctx, &data.data, env)
    }
}
