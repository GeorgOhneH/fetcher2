use druid::{Data, Widget, WidgetExt, WidgetPod, Lens, Point};
use druid::widget::prelude::*;
use druid::widget::{Checkbox, CrossAxisAlignment, Flex, Label};
use crate::ctypes::CType;
use crate::ctypes::option::COption;

impl Data for Box<COption> {
    fn same(&self, other: &Self) -> bool {
        self.as_ref().same(other.as_ref())
    }
}

impl COption {

    pub fn widget() -> impl Widget<Self> {
        Flex::column()
            .cross_axis_alignment(CrossAxisAlignment::Start)
            .with_child(CheckboxWrapper::new())
            .with_child(
                CType::widget().lens(Self::ty)
                    .disabled_if(|data: &Self, _env| !data.active),
            )
    }
}


struct CheckboxWrapper {
    checkbox: WidgetPod<bool, Checkbox>,
}

impl CheckboxWrapper {
    pub fn new() -> Self {
        Self {
            checkbox: WidgetPod::new(Checkbox::new("")),
        }
    }
}

impl Widget<COption> for CheckboxWrapper {
    fn event(&mut self, ctx: &mut EventCtx, event: &Event, data: &mut COption, env: &Env) {
        self.checkbox.event(ctx, event, &mut data.active, env)
    }

    fn lifecycle(
        &mut self,
        ctx: &mut LifeCycleCtx,
        event: &LifeCycle,
        data: &COption,
        env: &Env,
    ) {
        if let (LifeCycle::WidgetAdded, Some(name)) = (event, data.name) {
            self.checkbox
                .widget_mut()
                .set_text(name)
        }
        self.checkbox.lifecycle(ctx, event, &data.active, env)
    }

    fn update(
        &mut self,
        ctx: &mut UpdateCtx,
        _old_data: &COption,
        data: &COption,
        env: &Env,
    ) {
        self.checkbox.update(ctx, &data.active, env)
    }

    fn layout(
        &mut self,
        ctx: &mut LayoutCtx,
        bc: &BoxConstraints,
        data: &COption,
        env: &Env,
    ) -> Size {
        let size = self.checkbox.layout(ctx, bc, &data.active, env);
        self.checkbox
            .set_origin(ctx, &data.active, env, Point::ORIGIN);
        size
    }

    fn paint(&mut self, ctx: &mut PaintCtx, data: &COption, env: &Env) {
        self.checkbox.paint(ctx, &data.active, env)
    }
}
