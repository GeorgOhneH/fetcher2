use druid::{Point, WidgetPod};
use druid::widget::prelude::*;

pub struct InfoView<T, const N: usize> {
    current: Option<usize>,
    child_picker: Box<dyn Fn(&T, &Env) -> Option<usize>>,
    views: [WidgetPod<T, Box<dyn Widget<T>>>; N],
}

impl<T, const N: usize> InfoView<T, N> {
    pub fn new(
        child_picker: impl Fn(&T, &Env) -> Option<usize> + 'static,
        views: [Box<dyn Widget<T>>; N],
    ) -> Self {
        Self {
            current: None,
            child_picker: Box::new(child_picker),
            views: views.map(|widget| WidgetPod::new(widget)),
        }
    }
}

impl<T: Data, const N: usize> Widget<T> for InfoView<T, N> {
    fn event(&mut self, ctx: &mut EventCtx, event: &Event, data: &mut T, env: &Env) {
        if event.should_propagate_to_hidden() {
            for view in self.views.iter_mut() {
                view.event(ctx, event, data, env)
            }
        } else if let Some(current) = self.current {
            self.views[current].event(ctx, event, data, env)
        }
    }

    fn lifecycle(&mut self, ctx: &mut LifeCycleCtx, event: &LifeCycle, data: &T, env: &Env) {
        if let LifeCycle::WidgetAdded = event {
            let child_idx = (self.child_picker)(data, env);
            self.current = child_idx;
            ctx.children_changed();
        }
        if event.should_propagate_to_hidden() {
            for view in self.views.iter_mut() {
                view.lifecycle(ctx, event, data, env)
            }
        } else if let Some(current) = self.current {
            self.views[current].lifecycle(ctx, event, data, env)
        }
    }

    fn update(&mut self, ctx: &mut UpdateCtx, _: &T, data: &T, env: &Env) {
        let child_idx = (self.child_picker)(data, env);
        if child_idx != self.current {
            self.current = child_idx;
            ctx.children_changed();
        }
        if let Some(current) = self.current {
            self.views[current].update(ctx, data, env)
        }
    }

    fn layout(&mut self, ctx: &mut LayoutCtx, bc: &BoxConstraints, data: &T, env: &Env) -> Size {
        if let Some(current) = self.current {
            let size = self.views[current].layout(ctx, bc, data, env);
            self.views[current].set_origin(ctx, data, env, Point::ORIGIN);
            size
        } else {
            bc.max()
        }
    }

    fn paint(&mut self, ctx: &mut PaintCtx, data: &T, env: &Env) {
        if let Some(current) = self.current {
            self.views[current].paint(ctx, data, env)
        }
    }
}
