use druid::widget::Checkbox;
use druid::{
    BoxConstraints, Env, Event, EventCtx, LayoutCtx, LifeCycle, LifeCycleCtx, PaintCtx, Size,
    UpdateCtx, Widget,
};

use crate::ctypes::bool::CBool;

impl CBool {
    pub fn widget() -> impl Widget<Self> {
        CBoolWidget::new()
    }
}

pub struct CBoolWidget {
    checkbox: Checkbox,
}

impl CBoolWidget {
    pub fn new() -> Self {
        Self {
            checkbox: Checkbox::new(""),
        }
    }
}

impl Widget<CBool> for CBoolWidget {
    fn event(&mut self, ctx: &mut EventCtx, event: &Event, data: &mut CBool, env: &Env) {
        let mut value = data.value.unwrap_or(false);
        self.checkbox.event(ctx, event, &mut value, env);
        data.value = Some(value)
    }

    fn lifecycle(&mut self, ctx: &mut LifeCycleCtx, event: &LifeCycle, data: &CBool, env: &Env) {
        if let LifeCycle::WidgetAdded = event {
            if let Some(name) = data.name {
                self.checkbox.set_text(name)
            }
        }
        self.checkbox
            .lifecycle(ctx, event, &data.value.unwrap_or(false), env)
    }

    fn update(&mut self, ctx: &mut UpdateCtx, old_data: &CBool, data: &CBool, env: &Env) {
        self.checkbox.update(
            ctx,
            &old_data.value.unwrap_or(false),
            &data.value.unwrap_or(false),
            env,
        )
    }

    fn layout(
        &mut self,
        ctx: &mut LayoutCtx,
        bc: &BoxConstraints,
        data: &CBool,
        env: &Env,
    ) -> Size {
        self.checkbox
            .layout(ctx, bc, &data.value.unwrap_or(false), env)
    }

    fn paint(&mut self, ctx: &mut PaintCtx, data: &CBool, env: &Env) {
        self.checkbox.paint(ctx, &data.value.unwrap_or(false), env)
    }
}
