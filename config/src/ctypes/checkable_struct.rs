use crate::{CStruct, CStructBuilder, State};
use druid::widget::{Checkbox, Container, CrossAxisAlignment, Flex, Label, LabelText, List, Maybe};
use druid::{
    BoxConstraints, Color, Data, Env, Event, EventCtx, LayoutCtx, Lens, LifeCycle, LifeCycleCtx,
    PaintCtx, Point, Size, UpdateCtx, Widget, WidgetExt, WidgetPod,
};
use ron::Map;

#[derive(Debug, Clone, Data, Lens)]
pub struct CCheckableStruct {
    inner: CStruct,
    name: Option<String>, // CStruct always has no name
    checked: bool,
}

impl CCheckableStruct {
    fn new(config_struct: CStruct, checked: bool, name: Option<String>) -> Self {
        Self {
            inner: config_struct,
            checked,
            name,
        }
    }

    pub fn get_inner(&self) -> &CStruct {
        &self.inner
    }

    pub fn get_inner_mut(&mut self) -> &mut CStruct {
        &mut self.inner
    }

    pub fn set_checked(&mut self, checked: bool) {
        self.checked = checked;
    }

    pub fn is_checked(&self) -> bool {
        self.checked
    }

    pub fn state(&self) -> State {
        if self.checked {
            State::None
        } else {
            self.inner.state()
        }
    }

    pub fn widget() -> impl Widget<Self> {
        Flex::column()
            .cross_axis_alignment(CrossAxisAlignment::Start)
            .with_child(CheckboxWrapper::new())
            .with_child(
                CStruct::widget()
                    .lens(Self::inner)
                    .disabled_if(|data: &Self, env| !data.checked),
            )
    }
}

pub struct CCheckableStructBuilder {
    checked: Option<bool>,
    name: Option<String>,
    struct_builder: CStructBuilder,
}

impl CCheckableStructBuilder {
    pub fn new(struct_builder: CStructBuilder) -> Self {
        Self {
            struct_builder,
            checked: None,
            name: None,
        }
    }
    pub fn checked(mut self, checked: bool) -> Self {
        self.checked = Some(checked);
        self
    }

    pub fn name(mut self, name: String) -> Self {
        self.name = Some(name);
        self
    }

    pub fn build(self) -> CCheckableStruct {
        CCheckableStruct::new(
            self.struct_builder.build(),
            self.checked.unwrap_or(false),
            self.name,
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

impl Widget<CCheckableStruct> for CheckboxWrapper {
    fn event(&mut self, ctx: &mut EventCtx, event: &Event, data: &mut CCheckableStruct, env: &Env) {
        self.checkbox.event(ctx, event, &mut data.checked, env)
    }

    fn lifecycle(
        &mut self,
        ctx: &mut LifeCycleCtx,
        event: &LifeCycle,
        data: &CCheckableStruct,
        env: &Env,
    ) {
        if let (LifeCycle::WidgetAdded, Some(name)) = (event, &data.name) {
            self.checkbox
                .widget_mut()
                .set_text(LabelText::from(name.as_str()))
        }
        self.checkbox.lifecycle(ctx, event, &data.checked, env)
    }

    fn update(
        &mut self,
        ctx: &mut UpdateCtx,
        old_data: &CCheckableStruct,
        data: &CCheckableStruct,
        env: &Env,
    ) {
        self.checkbox.update(ctx, &data.checked, env)
    }

    fn layout(
        &mut self,
        ctx: &mut LayoutCtx,
        bc: &BoxConstraints,
        data: &CCheckableStruct,
        env: &Env,
    ) -> Size {
        let size = self.checkbox.layout(ctx, bc, &data.checked, env);
        self.checkbox
            .set_origin(ctx, &data.checked, env, Point::ORIGIN);
        size
    }

    fn paint(&mut self, ctx: &mut PaintCtx, data: &CCheckableStruct, env: &Env) {
        self.checkbox.paint(ctx, &data.checked, env)
    }
}
