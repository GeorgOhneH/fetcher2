#![allow(dead_code)]
#![feature(try_trait_v2)]
#![feature(control_flow_enum)]
#![feature(backtrace)]
#![allow(unused_imports)]

mod error;
mod session;
mod settings;
mod site_modules;
mod task;
mod template;

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

    let mut template = Template::new();
    tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .unwrap()
        .block_on(run(&mut template));
}

async fn run(template: &mut Template) {
    let session = crate::session::Session::new();
    let dsettings = Arc::new(DownloadSettings {
        username: std::env::var("USERNAME").unwrap(),
        password: std::env::var("PASSWORD").unwrap(),
        save_path: PathBuf::from("C:\\programming\\rust\\fetcher2\\test"),
        download_args: DownloadArgs {
            extensions: Extensions {
                inner: im::HashSet::new(),
                mode: Mode::Forbidden,
            },
            keep_old_files: true,
        },
        force: false,
    });
    let start = Instant::now();

    match template.prepare(&session, dsettings.clone()).await {
        Ok(()) => {}
        Err(err) => {
            print!("{:?}", err);
            println!("{}", err.backtrace().unwrap());
            return;
        }
    };

    match template.run_root(&session, dsettings).await {
        Ok(()) => {}
        Err(err) => {
            print!("{:?}", err);
            println!("{}", err.backtrace().unwrap());
            return;
        }
    };
    println!("{:#?}", start.elapsed());
    let save_path = PathBuf::from("C:\\programming\\rust\\fetcher2\\test.yml");
    template.save(&save_path).await.unwrap();
    // template.load(&save_path).await.unwrap();
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

use crate::template::widget::{TemplateData, TemplateWidget};
use crate::template::Node;
use config::CStruct;
use druid::im::{vector, Vector};
use druid::lens::{self, InArc, LensExt};
use druid::text::{Formatter, ParseFormatter, Selection, Validation, ValidationError};
use druid::widget::{
    Button, CrossAxisAlignment, Flex, Label, LineBreaking, List, Scroll, Spinner, Switch, TextBox,
};
use druid::{
    im, AppDelegate, AppLauncher, Color, Data, Env, Event, EventCtx, ExtEventSink, Handled,
    LayoutCtx, Lens, LifeCycle, LifeCycleCtx, LocalizedString, PaintCtx, Selector, SingleUse,
    Target, UnitPoint, UpdateCtx, Widget, WidgetExt, WidgetId, WidgetPod, WindowDesc,
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
    let delegate = Delegate::new(sink, template);
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
    // lists.add_child(
    //     Button::new("Start")
    //         .on_click(|ctx, data: &mut Arc<Template>, _| {
    //             main2(data);
    //             ctx.request_update()
    //         })
    //         .lens(AppData::template),
    // );

    lists.add_flex_child(
        Scroll::new(template).vertical().lens(AppData::template),
        1.0,
    );

    lists.with_child(Label::new("horizontal list")).with_child(
        FutureWidget::new(
            |_, _env| async {
                time::sleep(Duration::from_millis(5000)).await;
                2021
            },
            Flex::column()
                .with_child(Spinner::new())
                .with_spacer(10.0)
                .with_child(Label::new("Loading ...")),
            |value, data, _env| {
                // data is mut and value is owned
                Label::new(format!("Your number is {:?}", data)).boxed()
            },
        )
        .center(),
    )
    // .debug_paint_layout()
}

struct Request {
    future: Pin<Box<dyn Future<Output = Box<dyn Any + Send>> + Send>>,
    sender: WidgetId,
}

struct Response {
    value: Box<dyn Any + Send>,
}

const ASYNC_RESPONSE: Selector<SingleUse<Response>> = Selector::new("druid-async.async-response");
const SPAWN_ASYNC: Selector<SingleUse<Request>> = Selector::new("druid-async.spawn-async");

pub struct Delegate {
    tx: flume::Sender<Request>,
}

impl Delegate {
    pub fn new(sink: ExtEventSink, template: Template) -> Self {
        let (tx, rx) = flume::unbounded();
        thread::spawn(move || {
            other_thread(sink, rx, template);
        });
        Self { tx }
    }
}

impl<T: Data> AppDelegate<T> for Delegate {
    fn command(
        &mut self,
        _ctx: &mut druid::DelegateCtx,
        _target: druid::Target,
        cmd: &druid::Command,
        _data: &mut T,
        _env: &Env,
    ) -> Handled {
        if let Some(req) = cmd.get(SPAWN_ASYNC) {
            let req = req.take().expect("Someone stole our SPAWN_ASYNC command.");
            self.tx.send(req).unwrap();
            Handled::Yes
        } else {
            Handled::No
        }
    }
}

fn other_thread(sink: ExtEventSink, rx: flume::Receiver<Request>, mut template: Template) {
    tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .unwrap()
        .block_on(async move {
            let rx = rx.stream();
            let mut template = template;
            let session = crate::session::Session::new();
            let dsettings = Arc::new(DownloadSettings {
                username: std::env::var("USERNAME").unwrap(),
                password: std::env::var("PASSWORD").unwrap(),
                save_path: PathBuf::from("C:\\programming\\rust\\fetcher2\\test"),
                download_args: DownloadArgs {
                    extensions: Extensions {
                        inner: im::HashSet::new(),
                        mode: Mode::Forbidden,
                    },
                    keep_old_files: true,
                },
                force: false,
            });

            match template.prepare(&session, dsettings.clone()).await {
                Ok(()) => {}
                Err(err) => {
                    print!("{:?}", err);
                    println!("{}", err.backtrace().unwrap());
                    return;
                }
            };
            rx.for_each(|req| async {
                println!("GOT REQUEST");
                let sink = sink.clone();
                let res = req.future.await;
                match template.run_root(&session, dsettings.clone()).await {
                    Ok(()) => {}
                    Err(err) => {
                        print!("{:?}", err);
                        println!("{}", err.backtrace().unwrap());
                        return;
                    }
                };
                let res = Response { value: res };
                let sender = req.sender;

                sink.submit_command(ASYNC_RESPONSE, SingleUse::new(res), Target::Widget(sender))
                    .unwrap();
            })
            .await;
        });
}

pub type FutureWidgetAction<T> =
    Box<dyn FnOnce(&T, &Env) -> BoxFuture<'static, Box<dyn Any + Send>>>;
pub type FutureWidgetDone<T, U> = Box<dyn FnOnce(Box<U>, &mut T, &Env) -> Box<dyn Widget<T>>>;

pub struct FutureWidget<T, U> {
    future: Option<FutureWidgetAction<T>>,
    inner: WidgetPod<T, Box<dyn Widget<T>>>,
    on_done: Option<FutureWidgetDone<T, U>>,
}

impl<T, U> FutureWidget<T, U> {
    pub fn new<FMaker, Fut, Done>(
        future_maker: FMaker,
        pending: impl Widget<T> + 'static,
        on_done: Done,
    ) -> Self
    where
        U: Send + 'static,
        FMaker: FnOnce(&T, &Env) -> Fut + 'static,
        Fut: Future<Output = U> + 'static + Send,
        Done: FnOnce(Box<U>, &mut T, &Env) -> Box<dyn Widget<T>> + 'static,
    {
        Self {
            future: Some(Box::new(move |data, env| {
                let fut = future_maker(data, env);
                Box::pin(async move { Box::new(fut.await) as _ })
            })),
            inner: WidgetPod::new(Box::new(pending)),
            on_done: Some(Box::new(on_done)),
        }
    }
}

impl<T: Data, U: 'static> Widget<T> for FutureWidget<T, U> {
    fn event(&mut self, ctx: &mut EventCtx, event: &Event, data: &mut T, env: &Env) {
        // println!("{:?}", event);
        if let Event::Command(cmd) = event {
            if let Some(res) = cmd.get(ASYNC_RESPONSE) {
                let res = res.take().unwrap();
                let value = res.value.downcast::<U>().unwrap();
                let on_done = self.on_done.take().unwrap();
                self.inner = WidgetPod::new((on_done)(value, data, env));
                ctx.children_changed();
                ctx.request_update();
                ctx.request_layout();
                ctx.request_paint();
                return;
            }
            #[cfg(debug_assertions)]
            if cmd.is(SPAWN_ASYNC) {
                // SPAWN_ASYNC should always be handled by the delegate
                panic!("FutureWidget used without using druid_async::Delegate");
            }
        }
        self.inner.event(ctx, event, data, env);
    }

    fn lifecycle(&mut self, ctx: &mut LifeCycleCtx, event: &LifeCycle, data: &T, env: &Env) {
        if let LifeCycle::WidgetAdded = event {
            ctx.submit_command(SPAWN_ASYNC.with(SingleUse::new(Request {
                future: (self.future.take().unwrap())(data, env),
                sender: ctx.widget_id(),
            })));
        }
        self.inner.lifecycle(ctx, event, data, env)
    }

    fn update(&mut self, ctx: &mut UpdateCtx, _old_data: &T, data: &T, env: &Env) {
        self.inner.update(ctx, data, env)
    }

    fn layout(
        &mut self,
        ctx: &mut LayoutCtx,
        bc: &druid::BoxConstraints,
        data: &T,
        env: &Env,
    ) -> druid::Size {
        self.inner.layout(ctx, bc, data, env)
    }

    fn paint(&mut self, ctx: &mut PaintCtx, data: &T, env: &Env) {
        self.inner.paint(ctx, data, env)
    }
}
