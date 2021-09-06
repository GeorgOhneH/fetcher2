
use crate::delegate::{Msg, TemplateDelegate, MSG_THREAD};
use crate::template::widget::{TemplateData};
use config::{CStruct, Config};
use config::State;
use druid::im::{vector, Vector};
use druid::lens::{self, InArc, LensExt};
use druid::text::{Formatter, ParseFormatter, Selection, Validation, ValidationError};
use druid::widget::{
    Button, Controller, CrossAxisAlignment, Either, Flex, Label, LineBreaking, List, Maybe, Scroll,
    Spinner, Switch, TextBox,
};
use druid::{
    im, AppDelegate, AppLauncher, Application, Color, Command, Data, DelegateCtx, Env, Event,
    EventCtx, ExtEventSink, Handled, LayoutCtx, Lens, LifeCycle, LifeCycleCtx, LocalizedString,
    MouseButton, PaintCtx, Point, Screen, Selector, SingleUse, Size, Target, UnitPoint, UpdateCtx,
    Vec2, Widget, WidgetExt, WidgetId, WidgetPod, WindowConfig, WindowDesc, WindowLevel,
};
use druid_widget_nursery::Tree;
use flume;
use futures::future::BoxFuture;
use std::any::Any;
use std::cmp::max;
use std::collections::HashMap;
use std::future::Future;
use std::path::PathBuf;
use std::pin::Pin;
use tokio::time;
use tokio::time::Duration;

#[derive(Clone, Lens, Debug, Data)]
pub struct CStructWindow<T> {
    value: Option<T>,
    c_struct: Option<CStruct>,
}

impl<T> CStructWindow<T> {
    pub fn new() -> Self {
        Self {
            value: None,
            c_struct: None,
        }
    }
}

impl<T: Config + Data> CStructWindow<T> {
    pub fn widget() -> impl Widget<Self> {
        Flex::column()
            .with_flex_child(
                Maybe::new(|| CStruct::widget(), || Label::new("Loading..."))
                    .lens(CStructWindow::c_struct)
                    .scroll(),
                1.0,
            )
            .with_child(
                Flex::row().with_child(
                    Button::new("Save")
                        .on_click(|ctx, data: &mut CStructWindow<T>, env| {
                            if let Ok(settings) =
                            T::parse_from_app(data.c_struct.as_ref().unwrap())
                            {
                                data.value = Some(settings);
                                ctx.window().close();
                            }
                        })
                        .disabled_if(|data: &CStructWindow<T>, env| match &data.c_struct {
                            Some(c_struct) => !matches!(c_struct.state(), State::Valid),
                            None => true,
                        }),
                ).with_child(
                    Button::new("Cancel")
                        .on_click(|ctx, data: &mut CStructWindow<T>, env| {
                            ctx.window().close();
                        })),
            )
            .controller(WindowController::new())
    }
}

struct WindowController {}

impl WindowController {
    pub fn new() -> Self {
        Self {}
    }
}

impl<T: Config + Data, W: Widget<CStructWindow<T>>> Controller<CStructWindow<T>, W> for WindowController {
    fn event(
        &mut self,
        child: &mut W,
        ctx: &mut EventCtx,
        event: &Event,
        data: &mut CStructWindow<T>,
        env: &Env,
    ) {
        if let Event::WindowConnected = event {
            data.c_struct = Some(T::builder().name("Settings").build())
        };
        child.event(ctx, event, data, env)
    }
}
