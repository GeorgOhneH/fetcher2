#![allow(dead_code)]
#![feature(try_trait_v2)]
#![feature(control_flow_enum)]
#![feature(backtrace)]
#![allow(unused_imports)]

mod background_thread;
mod cstruct_window;
mod delegate;
mod error;
mod session;
mod settings;
mod site_modules;
mod task;
mod template;
mod utils;
pub mod widgets;

pub use error::{Result, TError};

use crate::settings::{DownloadSettings, Settings};
use crate::template::{DownloadArgs, Extensions, Mode, Template};
use config::{CBool, CInteger, CKwarg, CPath, CString, CType, Config};
use config_derive::Config;
use futures::StreamExt;
use log::{debug, error, info, log_enabled, Level};
use serde::Serialize;
use std::collections::HashSet;
use std::error::Error;
use std::ops::{Deref, DerefMut};
use std::path::Path;
use std::sync::{Arc, Mutex, RwLock};
use std::time::Instant;
use std::{io, thread};

fn main2() {
    // let mut base_config = fern::Dispatch::new();
    //
    // base_config = base_config.level(log::LevelFilter::Trace);
    //
    // let stdout_config = fern::Dispatch::new()
    //     .filter(|metadata| {
    //         !(metadata.target().starts_with("html5ever")
    //             || !metadata.target().starts_with("mio")
    //             || !metadata.target().starts_with("reqwest::connect"))
    //     })
    //     .format(|out, message, record| {
    //         out.finish(format_args!(
    //             "[{}][{}][{}] {}",
    //             chrono::Local::now().format("%H:%M"),
    //             record.target(),
    //             record.level(),
    //             message
    //         ))
    //     })
    //     .chain(io::stdout());
    //
    // base_config.chain(stdout_config).apply().unwrap();

    // let mut template = Template::new();
    // tokio::runtime::Builder::new_multi_thread()
    //     .enable_all()
    //     .build()
    //     .unwrap()
    //     .block_on(run(&mut template));
}

use config::ConfigEnum;
#[derive(Config, Serialize, Clone, Debug)]
struct Test {
    // #[config(default = true, gui_name = "Hello")]
    // hello: bool,
    // #[config()]
    // hello2: String,
    // #[config(default = 0, min = 0)]
    // int: isize,
    // path: PathBuf,
    #[config(ty = "Enum")]
    efsdfs: Test2,

    #[config(ty = "Struct")]
    efsd3rfs: Test3,
}

#[derive(Config, Serialize, Clone, Debug)]
enum Test2 {
    Hellfffffffffffffffo(String),
    Foo(String),
    Bar,
}

#[derive(Config, Serialize, Clone, Debug)]
struct Test3 {
    #[config(default = true)]
    hello: bool,
    #[config()]
    hello2: String,
    #[config(default = 0, min = 0)]
    int: isize,
    path: PathBuf,
}

use crate::cstruct_window::CStructWindow;
use crate::delegate::{Msg, TemplateDelegate, MSG_THREAD};
use crate::template::widget::{TemplateData, TemplateWidget};
use config::CStruct;
use config::State;
use druid::im::{vector, Vector};
use druid::lens::{self, InArc, LensExt};
use druid::text::{Formatter, ParseFormatter, Selection, Validation, ValidationError};
use druid::widget::{
    Button, Checkbox, Controller, CrossAxisAlignment, Either, Flex, Label, LineBreaking, List,
    Maybe, Scroll, Spinner, Switch, TextBox,
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
use crate::widgets::header::Header;

#[derive(Clone, Lens, Debug, Data)]
struct AppData {
    template: TemplateData,
    settings_window: CStructWindow<Settings>,
}

//
pub fn main() {
    // let mut base_config = fern::Dispatch::new();
    //
    // base_config = base_config.level(log::LevelFilter::Trace);
    //
    // let stdout_config = fern::Dispatch::new()
    //     .filter(|metadata| {
    //         !(metadata.target().starts_with("html5ever")
    //             || !metadata.target().starts_with("mio")
    //             || !metadata.target().starts_with("reqwest::connect"))
    //     })
    //     .format(|out, message, record| {
    //         out.finish(format_args!(
    //             "[{}][{}][{}] {}",
    //             chrono::Local::now().format("%H:%M"),
    //             record.target(),
    //             record.level(),
    //             message
    //         ))
    //     })
    //     .chain(io::stdout());
    //
    // base_config.chain(stdout_config).apply().unwrap();

    let mut cstruct = Test::builder().build();
    // let mut test: Test = Test::parse_from_app(&cstruct).unwrap();
    // test.efsdfs = Some(Test2::Bar);
    // test.update_app(&mut cstruct).unwrap();
    let mut template = Template::new();
    let (data, widget) = template.widget();
    let data = AppData {
        template: data,
        settings_window: CStructWindow::new(),
    };
    let main_window = WindowDesc::new(ui_builder(widget))
        .title(LocalizedString::new("list-demo-window-title").with_placeholder("List Demo"));
    let app_launcher = AppLauncher::with_window(main_window);
    let sink = app_launcher.get_external_handle();
    template.set_sink(sink.clone());
    let delegate = TemplateDelegate::new(sink, template);

    // use tracing_subscriber::prelude::*;
    // let filter_layer = tracing_subscriber::filter::LevelFilter::DEBUG;
    // let fmt_layer = tracing_subscriber::fmt::layer()
    //     // Display target (eg "my_crate::some_mod::submod") with logs
    //     .with_target(true);
    //
    // tracing_subscriber::registry()
    //     .with(filter_layer)
    //     .with(fmt_layer)
    //     .init();

    app_launcher
        // .delegate(delegate)
        .log_to_console()
        .launch(data)
        .expect("launch failed");
}

fn ui_builder(template: TemplateWidget) -> impl Widget<AppData> {
    let header = Header::columns([Label::new("Hello"), Label::new("Hello2"), Label::new("Hello3")]).draggable(true);
    let start = Button::new("Start").on_click(|ctx, _, _| {
        ctx.submit_command(Command::new(
            MSG_THREAD,
            SingleUse::new(Msg::StartAll),
            Target::Global,
        ));
        // ctx.submit_command(Command::new(MSG_THREAD, SingleUse::new(Msg::Cancel), Target::Global))
    });
    let stop = Button::new("Stop").on_click(|ctx, _, _| {
        ctx.submit_command(Command::new(
            MSG_THREAD,
            SingleUse::new(Msg::Cancel),
            Target::Global,
        ))
    });
    let settings = Button::new("Settings")
        .on_click(|ctx, data: &mut CStructWindow<Settings>, env| {
            let window = ctx.window();
            let win_pos = window.get_position();
            let (win_size_w, win_size_h) = window.get_size().into();
            let (size_w, size_h) = (f64::min(600., win_size_w), f64::min(600., win_size_h));
            let pos = win_pos + ((win_size_w - size_w) / 2., (win_size_h - size_h) / 2.);
            ctx.new_sub_window(
                WindowConfig::default()
                    .show_titlebar(true)
                    .window_size(Size::new(size_w, size_h))
                    .set_position(pos)
                    .set_level(WindowLevel::Modal),
                CStructWindow::widget(),
                data.clone(),
                env.clone(),
            );
        })
        .padding(0.) // So it's enclosed in a WidgetPod, (just a nop)
        .lens(AppData::settings_window);

    Flex::column()
        .cross_axis_alignment(CrossAxisAlignment::Start)
        .with_child(start)
        .with_child(stop)
        .with_child(settings)
        .with_child(header)
        .with_flex_child(Scroll::new(template).vertical().lens(AppData::template), 1.)
    .debug_paint_layout()
}

#[derive(Clone, Lens, Debug, Data)]
struct Hello {
    vec: Vector<bool>,
}

fn main8() {
    let main_window = WindowDesc::new(hello2());
    AppLauncher::with_window(main_window)
        .log_to_console()
        .launch(Hello {
            vec: vec![true, false].into(),
        })
        .expect("launch failed");
}

fn hello3() -> impl Widget<Vector<bool>> {
    Flex::column()
        .with_child(List::new(|| Checkbox::new("Hello").center()))
        .with_child(
            Button::new("Add")
                .on_click(|_, c_vec: &mut Vector<bool>, _env| c_vec.push_back(false))
        )
}
fn hello2() -> impl Widget<Hello> {
    Flex::column()
        .with_child(hello3())
        .with_child(
            Button::new("Sub Window").on_click(|ctx, data: &mut Vector<bool>, env| {
                let window = ctx.window();
                let win_pos = window.get_position();
                let (win_size_w, win_size_h) = window.get_size().into();
                let (size_w, size_h) = (f64::min(600., win_size_w), f64::min(600., win_size_h));
                let pos = win_pos + ((win_size_w - size_w) / 2., (win_size_h - size_h) / 2.);
                ctx.new_sub_window(
                    WindowConfig::default()
                        .show_titlebar(true)
                        .window_size(Size::new(size_w, size_h))
                        .set_position(pos)
                        .set_level(WindowLevel::Modal),
                    hello3(),
                    data.clone(),
                    env.clone(),
                );
            })
                .padding(0.) // So it's enclosed in a WidgetPod, (just a nop),
        )
        .lens(Hello::vec)
}
