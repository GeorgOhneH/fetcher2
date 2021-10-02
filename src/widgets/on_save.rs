use druid::{Data, Env, Event, EventCtx, LifeCycleCtx, UpdateCtx, Widget};
use druid::widget::Controller;
use tracing::{instrument, trace};

pub struct Save<T, W> {
    init: Box<dyn Fn(&mut W, &mut LifeCycleCtx, &T, &Env)>,
    save: Box<dyn Fn(&mut W, &mut EventCtx, &mut T, &Env)>,
}

impl<T: Data, W: Widget<T>> Save<T, W> {
    pub fn new(
        init: impl Fn(&mut W, &mut LifeCycleCtx, &T, &Env) + 'static,
        save: impl Fn(&mut W, &mut EventCtx, &mut T, &Env) + 'static,
    ) -> Self {
        Self {
            init: Box::new(init),
            save: Box::new(save),
        }
    }
}

impl<T: Data, W: Widget<T>> Controller<T, W> for Save<T, W> {
    fn event(&mut self, child: &mut W, ctx: &mut EventCtx, event: &Event, data: &mut T, env: &Env) {
        if let Event::WindowCloseRequested = event {
            (self.save)(child, ctx, data, env);
        }
        child.event(ctx, event, data, env)
    }
    fn lifecycle(
        &mut self,
        child: &mut W,
        ctx: &mut LifeCycleCtx,
        event: &crate::LifeCycle,
        data: &T,
        env: &Env,
    ) {
        if let crate::LifeCycle::WidgetAdded = event {
            (self.init)(child, ctx, data, env);
        }
        child.lifecycle(ctx, event, data, env)
    }
}
