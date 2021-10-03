use druid::widget::Label;
use druid::Color;
use druid::{
    BoxConstraints, Env, Event, EventCtx, LayoutCtx, LifeCycle, LifeCycleCtx, PaintCtx, Size,
    UpdateCtx,
};
use druid::{Data, Widget};

pub struct WarningLabel<T> {
    label: Label<()>,
    active: bool,
    msg_fn: Box<dyn Fn(&T) -> Option<String>>,
}

impl<T> WarningLabel<T> {
    pub fn new(msg_fn: impl Fn(&T) -> Option<String> + 'static) -> Self {
        Self {
            label: Label::new("test").with_text_color(Color::rgb8(255, 0, 0)),
            active: true,
            msg_fn: Box::new(msg_fn),
        }
    }
}

impl<T: Data> Widget<T> for WarningLabel<T> {
    fn event(&mut self, ctx: &mut EventCtx, event: &Event, _data: &mut T, env: &Env) {
        self.label.event(ctx, event, &mut (), env);
    }

    fn lifecycle(&mut self, ctx: &mut LifeCycleCtx, event: &LifeCycle, data: &T, env: &Env) {
        if let LifeCycle::WidgetAdded = event {
            match (self.msg_fn)(data) {
                None => self.active = false,
                Some(msg) => {
                    self.active = true;
                    self.label.set_text(msg)
                }
            }
            ctx.request_layout();
        }
        self.label.lifecycle(ctx, event, &(), env)
    }

    fn update(&mut self, ctx: &mut UpdateCtx, old_data: &T, data: &T, env: &Env) {
        if !old_data.same(data) {
            match (self.msg_fn)(data) {
                None => self.active = false,
                Some(msg) => {
                    self.active = true;
                    self.label.set_text(msg);
                }
            }
            ctx.request_layout();
        }
        self.label.update(ctx, &(), &(), env);
    }

    fn layout(&mut self, ctx: &mut LayoutCtx, bc: &BoxConstraints, _data: &T, env: &Env) -> Size {
        if self.active {
            self.label.layout(ctx, bc, &(), env)
        } else {
            bc.min()
        }
    }

    fn paint(&mut self, ctx: &mut PaintCtx, _data: &T, env: &Env) {
        if self.active {
            self.label.paint(ctx, &(), env);
        }
    }
}
