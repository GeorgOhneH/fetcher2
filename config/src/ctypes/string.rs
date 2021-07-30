use crate::*;
use druid::widget::{Flex, Label, Maybe, TextBox};
use druid::{
    BoxConstraints, Data, Env, Event, EventCtx, LayoutCtx, Lens, LensExt, LifeCycle, LifeCycleCtx,
    PaintCtx, Size, UpdateCtx, Widget, WidgetExt,
};

#[derive(Debug, Clone, Data, Lens)]
pub struct CString {
    value: Option<String>,
    name: Option<String>,
}

impl CString {
    fn new() -> Self {
        Self {
            value: None,
            name: None,
        }
    }

    pub fn get(&self) -> Option<&String> {
        Option::from(&self.value)
    }

    pub fn set(&mut self, value: String) {
        self.value = Some(value);
    }
    pub fn unset(&mut self) {
        self.value = None
    }

    pub fn state(&self) -> State {
        match &self.value {
            Some(_) => State::Valid,
            None => State::None,
        }
    }

    pub fn widget() -> impl Widget<Self> {
        Flex::row()
            .with_child(
                Maybe::or_empty(|| Label::dynamic(|data: &String, _| data.clone() + ":"))
                    .lens(Self::name),
            )
            .with_child(
                Label::dynamic(|data: &Option<String>, _env| format!("{:?}", data))
                    .lens(Self::value),
            )
            .with_child(CStringWidget::new().lens(Self::value))
    }
}

pub struct CStringWidget {
    text_box: TextBox<String>,
    current_data: Option<String>,
}

impl CStringWidget {
    pub fn new() -> Self {
        Self {
            text_box: TextBox::new(),
            current_data: None,
        }
    }
}

impl Widget<Option<String>> for CStringWidget {
    fn event(&mut self, ctx: &mut EventCtx, event: &Event, data: &mut Option<String>, env: &Env) {
        match data {
            Some(str) => {
                self.text_box.event(ctx, event, str, env);
                if str.is_empty() {
                    *data = None
                }
            }
            None => {
                let mut new_data = "".to_owned();
                self.text_box.event(ctx, event, &mut new_data, env);
                if !new_data.is_empty() {
                    *data = Some(new_data)
                }
            }
        };
        self.current_data = data.clone();
        // dbg!(data);
    }

    fn lifecycle(
        &mut self,
        ctx: &mut LifeCycleCtx,
        event: &LifeCycle,
        data: &Option<String>,
        env: &Env,
    ) {
        self.text_box
            .lifecycle(ctx, event, data.as_ref().unwrap_or(&"".to_owned()), env)
    }

    fn update(
        &mut self,
        ctx: &mut UpdateCtx,
        old_data: &Option<String>,
        data: &Option<String>,
        env: &Env,
    ) {
        self.text_box.update(
            ctx,
            old_data.as_ref().unwrap_or(&"".to_owned()),
            data.as_ref().unwrap_or(&"".to_owned()),
            env,
        );
    }

    fn layout(
        &mut self,
        ctx: &mut LayoutCtx,
        bc: &BoxConstraints,
        data: &Option<String>,
        env: &Env,
    ) -> Size {
        self.text_box
            .layout(ctx, bc, data.as_ref().unwrap_or(&"".to_owned()), env)
    }

    fn paint(&mut self, ctx: &mut PaintCtx, data: &Option<String>, env: &Env) {
        self.text_box
            .paint(ctx, data.as_ref().unwrap_or(&"".to_owned()), env)
    }
}

pub struct CStringBuilder {
    inner: CString,
}

impl CStringBuilder {
    pub fn new() -> Self {
        Self {
            inner: CString::new(),
        }
    }

    pub fn default(mut self, value: String) -> Self {
        self.inner.set(value);
        self
    }

    pub fn name(mut self, name: String) -> Self {
        self.inner.name = Some(name);
        self
    }

    pub fn build(self) -> CString {
        self.inner
    }
}
