use druid::widget::prelude::*;
use druid::{Point, WidgetExt, WidgetPod};

pub struct Lazy<T> {
    maker: Box<dyn Fn() -> Box<dyn Widget<T>>>,
    widget: Option<WidgetPod<T, Box<dyn Widget<T>>>>,
}

impl<T: Data> Lazy<T> {
    pub fn new<W>(maker: impl Fn() -> W + 'static) -> Lazy<T>
    where
        W: Widget<T> + 'static,
    {
        Lazy {
            maker: Box::new(move || maker().boxed()),
            widget: None,
        }
    }
}

impl<T: Data> Widget<T> for Lazy<T> {
    fn event(&mut self, ctx: &mut EventCtx, event: &Event, data: &mut T, env: &Env) {
        self.widget.as_mut().unwrap().event(ctx, event, data, env)
    }

    fn lifecycle(&mut self, ctx: &mut LifeCycleCtx, event: &LifeCycle, data: &T, env: &Env) {
        if let LifeCycle::WidgetAdded = event {
            self.widget = Some(WidgetPod::new((*self.maker)().boxed()));
            ctx.children_changed()
        }
        self.widget
            .as_mut()
            .unwrap()
            .lifecycle(ctx, event, data, env)
    }

    fn update(&mut self, ctx: &mut UpdateCtx, _old_data: &T, data: &T, env: &Env) {
        self.widget.as_mut().unwrap().update(ctx, data, env)
    }

    fn layout(&mut self, ctx: &mut LayoutCtx, bc: &BoxConstraints, data: &T, env: &Env) -> Size {
        let s = self.widget.as_mut().unwrap().layout(ctx, bc, data, env);
        self.widget
            .as_mut()
            .unwrap()
            .set_origin(ctx, data, env, Point::ORIGIN);
        s
    }

    fn paint(&mut self, ctx: &mut PaintCtx, data: &T, env: &Env) {
        self.widget.as_mut().unwrap().paint(ctx, data, env)
    }
}
