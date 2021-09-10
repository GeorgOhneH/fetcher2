
use crate::delegate::{Msg, TemplateDelegate, MSG_THREAD};
use crate::template::widget_data::{TemplateData};
use config::{CStruct, Config};
use config::State;
use druid::im::{vector, Vector};
use druid::lens::{self, InArc, LensExt};
use druid::text::{Formatter, ParseFormatter, Selection, Validation, ValidationError};
use druid::widget::{
    Button, Controller, CrossAxisAlignment, Either, Flex, Label, LineBreaking, List, Maybe, Scroll,
    Spinner, Switch, TextBox,
};
use druid::{im, AppDelegate, AppLauncher, Application, Color, Command, Data, DelegateCtx, Env, Event, EventCtx, ExtEventSink, Handled, LayoutCtx, Lens, LifeCycle, LifeCycleCtx, LocalizedString, MouseButton, PaintCtx, Point, Screen, Selector, SingleUse, Size, Target, UnitPoint, UpdateCtx, Vec2, Widget, WidgetExt, WidgetId, WidgetPod, WindowConfig, WindowDesc, WindowLevel, BoxConstraints};
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
use std::marker::PhantomData;

use druid_widget_nursery::selectors;


selectors! {
    APPLY
}

pub struct CStructBuffer<T> {
    pub child: WidgetPod<CStruct, Box<dyn Widget<CStruct>>>,
    pub c_struct_data: CStruct,
    _marker: PhantomData<T>,
}

impl<T: Config + Data> CStructBuffer<T> {
    pub fn new() -> Self {
        Self {
            child: WidgetPod::new(CStruct::widget().boxed()),
            c_struct_data: T::builder().name("Settings").build(),
            _marker: PhantomData,
        }
    }
}

impl<T: Config + Data> Widget<Option<T>> for CStructBuffer<T> {
    fn event(&mut self, ctx: &mut EventCtx, event: &Event, data: &mut Option<T>, env: &Env) {
        self.child.event(ctx, event, &mut self.c_struct_data, env);
        match event {
            Event::Command(command) if command.is(APPLY) => {
                if let Ok(settings) =
                T::parse_from_app(&self.c_struct_data)
                {
                    *data = Some(settings);
                    ctx.window().close();
                }
            }
            _ => (),
        }
    }

    fn lifecycle(&mut self, ctx: &mut LifeCycleCtx, event: &LifeCycle, data: &Option<T>, env: &Env) {
        if let LifeCycle::WidgetAdded = event {
            if let Some(init) = data {
                init.clone().update_app(&mut self.c_struct_data).unwrap();
            }
        }
        self.child.lifecycle(ctx, event, &self.c_struct_data, env)
    }

    fn update(&mut self, ctx: &mut UpdateCtx, old_data: &Option<T>, data: &Option<T>, env: &Env) {
        self.child.update(ctx, &self.c_struct_data, env)
    }

    fn layout(&mut self, ctx: &mut LayoutCtx, bc: &BoxConstraints, data: &Option<T>, env: &Env) -> Size {
        let size = self.child.layout(ctx, bc, &self.c_struct_data, env);
        self.child.set_origin(ctx, &self.c_struct_data, env, Point::ORIGIN);
        size
    }

    fn paint(&mut self, ctx: &mut PaintCtx, data: &Option<T>, env: &Env) {
        self.child.paint(ctx, &self.c_struct_data, env)
    }
}


pub fn c_struct_window<T: Config + Data>() -> impl Widget<Option<T>> {
    Flex::column()
        .with_flex_child(
            CStructBuffer::new().scroll(),
            1.0,
        )
        .with_child(
            Flex::row().with_child(
                Button::new("Save")
                    .on_click(|ctx, data: &mut Option<T>, env| {
                        ctx.submit_command(APPLY.to(Target::Window(ctx.window_id())));
                    })
                    // .disabled_if(|data: &Option<T>, env| match &data.c_struct {
                    //     Some(c_struct) => !matches!(c_struct.state(), State::Valid),
                    //     None => true,
                    // }),
            ).with_child(
                Button::new("Cancel")
                    .on_click(|ctx, data: &mut Option<T>, env| {
                        ctx.window().close();
                    })),
        )
}


