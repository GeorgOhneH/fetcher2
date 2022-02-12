use druid::widget::{Flex, Label, Maybe, TextBox};
use druid::{
    BoxConstraints, Env, Event, EventCtx, LayoutCtx, LifeCycle, LifeCycleCtx, PaintCtx, Point,
    Size, UpdateCtx, Widget, WidgetExt, WidgetPod,
};

use crate::ctypes::string::CString;

impl CString {
    pub fn widget() -> impl Widget<Self> {
        Flex::row()
            .with_child(
                Maybe::or_empty(|| Label::dynamic(|data: &&'static str, _| format!("{data}:")))
                    .lens(Self::name),
            )
            .with_child(CStringWidget::new().lens(Self::value))
    }
}

pub struct CStringWidget {
    text_box: WidgetPod<String, TextBox<String>>,
}

impl CStringWidget {
    pub fn new() -> Self {
        Self {
            text_box: WidgetPod::new(TextBox::new()),
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
        _old_data: &Option<String>,
        data: &Option<String>,
        env: &Env,
    ) {
        self.text_box
            .update(ctx, data.as_ref().unwrap_or(&"".to_owned()), env);
    }

    fn layout(
        &mut self,
        ctx: &mut LayoutCtx,
        bc: &BoxConstraints,
        data: &Option<String>,
        env: &Env,
    ) -> Size {
        let size = self
            .text_box
            .layout(ctx, bc, data.as_ref().unwrap_or(&"".to_owned()), env);
        self.text_box.set_origin(
            ctx,
            data.as_ref().unwrap_or(&"".to_owned()),
            env,
            Point::ORIGIN,
        );
        size
    }

    fn paint(&mut self, ctx: &mut PaintCtx, data: &Option<String>, env: &Env) {
        self.text_box
            .paint(ctx, data.as_ref().unwrap_or(&"".to_owned()), env)
    }
}
