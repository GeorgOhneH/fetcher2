use druid::widget::Checkbox;
use druid::{
    BoxConstraints, Data, Env, Event, EventCtx, LayoutCtx, LifeCycle, LifeCycleCtx, PaintCtx, Size,
    UpdateCtx, Widget,
};

use crate::State;

#[derive(Debug, Clone, Data)]
pub struct CBool {
    value: bool,
    #[data(ignore)]
    name: Option<String>,
}

impl CBool {
    fn new() -> Self {
        Self {
            value: false,
            name: None,
        }
    }
    pub fn get(&self) -> bool {
        self.value
    }

    pub fn set_option(&mut self, value: Option<bool>) {
        self.value = value.unwrap_or(false);
    }

    pub fn set(&mut self, value: bool) {
        self.value = value;
    }
    pub fn unset(&mut self) {
        self.value = false
    }

    pub fn widget() -> impl Widget<Self> {
        CBoolWidget::new()
    }

    pub fn state(&self) -> State {
        State::Valid
    }
}

pub struct CBoolBuilder {
    inner: CBool,
}

impl CBoolBuilder {
    pub fn new() -> Self {
        Self {
            inner: CBool::new(),
        }
    }
    pub fn default(mut self, value: bool) -> Self {
        self.inner.set(value);
        self
    }

    pub fn name(mut self, name: String) -> Self {
        self.inner.name = Some(name);
        self
    }

    pub fn build(self) -> CBool {
        self.inner
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
        self.checkbox.event(ctx, event, &mut data.value, env)
    }

    fn lifecycle(&mut self, ctx: &mut LifeCycleCtx, event: &LifeCycle, data: &CBool, env: &Env) {
        if let LifeCycle::WidgetAdded = event {
            if let Some(name) = &data.name {
                self.checkbox.set_text(name.clone())
            }
        }
        self.checkbox.lifecycle(ctx, event, &data.value, env)
    }

    fn update(&mut self, ctx: &mut UpdateCtx, old_data: &CBool, data: &CBool, env: &Env) {
        self.checkbox.update(ctx, &old_data.value, &data.value, env)
    }

    fn layout(
        &mut self,
        ctx: &mut LayoutCtx,
        bc: &BoxConstraints,
        data: &CBool,
        env: &Env,
    ) -> Size {
        self.checkbox.layout(ctx, bc, &data.value, env)
    }

    fn paint(&mut self, ctx: &mut PaintCtx, data: &CBool, env: &Env) {
        self.checkbox.paint(ctx, &data.value, env)
    }
}
