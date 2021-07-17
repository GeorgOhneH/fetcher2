#![allow(dead_code)]
#![feature(try_trait_v2)]
#![feature(control_flow_enum)]
#![feature(backtrace)]
#![allow(unused_imports)]

mod background_thread;
mod delegate;
mod error;
mod session;
mod settings;
mod site_modules;
mod task;
mod template;
mod utils;

pub use error::{Result, TError};

use crate::settings::DownloadSettings;
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
    #[config(ty = "enum")]
    efsdfs: Test2,

    #[config(ty = "struct")]
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

use crate::delegate::{Msg, TemplateDelegate, MSG_THREAD};
use crate::template::widget::{TemplateData, TemplateWidget};
use config::CStruct;
use druid::im::{vector, Vector};
use druid::lens::{self, InArc, LensExt};
use druid::text::{Formatter, ParseFormatter, Selection, Validation, ValidationError};
use druid::widget::{
    Button, CrossAxisAlignment, Flex, Label, LineBreaking, List, Scroll, Spinner, Switch, TextBox,
};
use druid::{
    im, AppDelegate, AppLauncher, Color, Command, Data, DelegateCtx, Env, Event, EventCtx,
    ExtEventSink, Handled, LayoutCtx, Lens, LifeCycle, LifeCycleCtx, LocalizedString, PaintCtx,
    Selector, SingleUse, Target, UnitPoint, UpdateCtx, Widget, WidgetExt, WidgetId, WidgetPod,
    WindowDesc,
};
use druid_widget_nursery::Tree;
use flume;
use futures::future::BoxFuture;
use std::any::Any;
use std::collections::HashMap;
use std::future::Future;
use std::path::PathBuf;
use std::pin::Pin;
use tokio::time;
use tokio::time::Duration;

#[derive(Clone, Lens, Debug, Data)]
struct AppData {
    template: TemplateData,
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
    let data = AppData { template: data };
    let main_window = WindowDesc::new(ui_builder(widget))
        .title(LocalizedString::new("list-demo-window-title").with_placeholder("List Demo"));
    let app_launcher = AppLauncher::with_window(main_window);
    let sink = app_launcher.get_external_handle();
    template.set_sink(sink.clone());
    let delegate = TemplateDelegate::new(sink, template);
    app_launcher
        .delegate(delegate)
        .log_to_console()
        .launch(data)
        .expect("launch failed");
}

fn ui_builder(template: TemplateWidget) -> impl Widget<AppData> {
    let mut lists = Flex::column().cross_axis_alignment(CrossAxisAlignment::Start);

    lists.add_child(
        Label::dynamic(|data, _env| format!("{:?}", data))
            .with_line_break_mode(LineBreaking::WordWrap),
    );
    lists.add_child(Button::new("Start").on_click(|ctx, _, _| {
        ctx.submit_command(Command::new(
            MSG_THREAD,
            SingleUse::new(Msg::StartAll),
            Target::Global,
        ));
        // ctx.submit_command(Command::new(MSG_THREAD, SingleUse::new(Msg::Cancel), Target::Global))
    }));
    lists.add_child(Button::new("Stop").on_click(|ctx, _, _| {
        ctx.submit_command(Command::new(
            MSG_THREAD,
            SingleUse::new(Msg::Cancel),
            Target::Global,
        ))
    }));

    lists.add_flex_child(
        Scroll::new(template).vertical().lens(AppData::template),
        1.0,
    );

    lists
    // .debug_paint_layout()
}
