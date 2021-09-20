#![allow(dead_code)]
#![feature(try_trait_v2)]
#![feature(control_flow_enum)]
#![feature(backtrace)]
#![feature(type_alias_impl_trait)]
#![allow(unused_imports)]

use std::any::Any;
use std::cmp::max;
use std::collections::HashMap;
use std::collections::HashSet;
use std::error::Error;
use std::future::Future;
use std::ops::{Deref, DerefMut};
use std::path::Path;
use std::path::PathBuf;
use std::pin::Pin;
use std::sync::{Arc, Mutex, RwLock};
use std::time::Instant;
use std::{fs, io, thread};

use config::CStruct;
use config::State;
use config::{CBool, CInteger, CKwarg, CPath, CString, CType, Config};
use druid::im::{vector, Vector};
use druid::lens::{self, InArc, LensExt};
use druid::text::{Formatter, ParseFormatter, Selection, Validation, ValidationError};
use druid::widget::{
    Button, Checkbox, Controller, CrossAxisAlignment, Either, Flex, Label, LineBreaking, List,
    Maybe, Scroll, SizedBox, Spinner, Switch, TextBox,
};
use druid::{
    im, AppDelegate, AppLauncher, Application, Color, Command, Data, DelegateCtx, Env, Event,
    EventCtx, ExtEventSink, Handled, LayoutCtx, Lens, LifeCycle, LifeCycleCtx, LocalizedString,
    Menu, MenuItem, MouseButton, PaintCtx, Point, Screen, Selector, SingleUse, Size, Target,
    UnitPoint, UpdateCtx, Vec2, Widget, WidgetExt, WidgetId, WidgetPod, WindowConfig, WindowDesc,
    WindowLevel,
};
use flume;
use futures::future::BoxFuture;
use futures::StreamExt;
use log::{debug, error, info, log_enabled, Level};
use serde::Serialize;
use tokio::time;
use tokio::time::Duration;

pub use error::{Result, TError};

use crate::background_thread::background_main;
use crate::controller::{MainController, Msg, WINDOW_STATE_DIR};
use crate::cstruct_window::CStructBuffer;
use crate::settings::{DownloadSettings, Settings, Test};
use crate::template::communication::RawCommunication;
use crate::template::nodes::node_data::NodeData;
use crate::template::nodes::root_data::RootNodeData;
use crate::template::widget_data::TemplateData;
use crate::template::{DownloadArgs, Extensions, Mode, Template};
use crate::ui::{build_ui, make_menu, AppData, TemplateInfoSelect};
use crate::widgets::file_watcher::FileWatcher;
use crate::widgets::header::Header;
use crate::widgets::tree::Tree;
use std::io::Write;

mod background_thread;
pub mod controller;
mod cstruct_window;
pub mod edit_window;
mod error;
mod session;
mod settings;
mod site_modules;
mod task;
mod template;
pub mod ui;
mod utils;
pub mod widgets;

fn load_window_state() -> Result<AppData> {
    let file_content = &fs::read(WINDOW_STATE_DIR.as_path())?;
    Ok(ron::de::from_bytes::<AppData>(file_content)?)
}
fn save_window_state(app_state: &AppData) -> Result<()> {
    let serialized = ron::to_string(app_state)?;

    fs::create_dir_all(WINDOW_STATE_DIR.as_path().parent().expect(""))?;

    let mut f = fs::OpenOptions::new()
        .write(true)
        .truncate(true)
        .create(true)
        .open(WINDOW_STATE_DIR.as_path())?;
    f.write_all(&serialized.as_bytes())?;
    Ok(())
}

fn build_window(load_err: Option<TError>, tx: flume::Sender<Msg>) -> WindowDesc<AppData> {
    let mut main_window = WindowDesc::new(
        build_ui()
            .controller(MainController::new(load_err, tx))
            .padding(0.),
    )
    .menu(make_menu)
    .title(LocalizedString::new("list-demo-window-title").with_placeholder("List Demo"));
    main_window
}

pub fn main() {
    let (tx, rx) = flume::unbounded();
    let (s, r) = crossbeam_channel::bounded(5);
    let handle = thread::spawn(move || {
        background_main(rx, r);
    });

    AppData::default().expect("AppData should always have a default");

    let (app_data, load_err) = match load_window_state() {
        Ok(data) => (data, None),
        Err(err) => (
            AppData::default().expect("AppData should always have a default"),
            Some(err),
        ),
    };
    let main_window = build_window(load_err, tx.clone());

    let app_launcher = AppLauncher::with_window(main_window);

    let sink = app_launcher.get_external_handle();
    s.send(sink).unwrap();

    use tracing_subscriber::prelude::*;
    let filter_layer = tracing_subscriber::filter::LevelFilter::DEBUG;
    let filter = tracing_subscriber::filter::EnvFilter::default()
        .add_directive("my_crate=trace".parse().unwrap())
        .add_directive("druid=trace".parse().unwrap())
        .add_directive("druid_widget_nursery=trace".parse().unwrap());
    let fmt_layer = tracing_subscriber::fmt::layer()
        // Display target (eg "my_crate::some_mod::submod") with logs
        .with_target(true);

    tracing_subscriber::registry()
        .with(filter_layer)
        .with(fmt_layer)
        .with(filter)
        .init();

    app_launcher
        // .log_to_console()
        .launch(app_data)
        .expect("launch failed");

    tx.send(Msg::ExitAndSave).expect("");
    // let _ = save_window_state(&app_state.read().unwrap());
    handle.join().expect("thread panicked");
}
//
// use config::{CStruct, Config, ConfigEnum};
// use ron::ser::PrettyConfig;
// use ron::{Map, Value};
// use serde::Serialize;
// use serde::__private::fmt::Debug;
// use std::collections::HashMap;
// use std::marker::PhantomData;
// use std::path::PathBuf;
// use std::sync::{Arc, Mutex, RwLock};
//
// //
// fn main() {
//     #[derive(Debug, Config, PartialEq, Clone)]
//     struct Hello {
//         bool: bool,
//         bool_option: Option<bool>,
//         int: isize,
//         int_option: Option<isize>,
//         str: String,
//         str_option: Option<String>,
//         path: PathBuf,
//         path_option: Option<PathBuf>,
//         normal_map: HashMap<String, Vec<String>>,
//         #[config(ty = "_<_, struct>")]
//         path_map: HashMap<PathBuf, NestedStruct>,
//
//         vec: Vec<isize>,
//         #[config(ty = "_<struct>")]
//         vec_nested: Vec<NestedStruct>,
//
//         #[config(ty = "struct")]
//         nested: NestedStruct,
//         #[config(ty = "_<struct>")]
//         nested_option: Option<NestedStruct>,
//
//         #[config(ty = "enum")]
//         test_enum: HelloEnum,
//
//         #[config(ty = "_<enum>")]
//         test_enum_option: Option<HelloEnum>,
//
//         wrapper: Arc<String>,
//
//         #[config(ty = "struct")]
//         tstruct: TStruct<NestedStruct>,
//
//         #[config(ty = "_<struct>")]
//         tstruct_option: Option<TStruct<NestedStruct>>,
//     }
//
//     #[derive(Debug, ConfigEnum, PartialEq, Clone)]
//     enum HelloEnum {
//         Unit,
//         #[config(ty = "struct")]
//         Struct(NestedStruct),
//         With(String),
//     }
//
//     #[derive(Debug, Config, PartialEq, Clone)]
//     struct NestedStruct {
//         x: isize,
//     }
//
//     //
//     let mut map = HashMap::new();
//     map.insert("Hello".to_string(), vec!["hello again".to_string()]);
//     let mut map2 = HashMap::new();
//     map2.insert(PathBuf::from("Hello"), NestedStruct { x: 42 });
//
//     let init = Hello {
//         bool: true,
//         bool_option: Some(false),
//         int: -10,
//         int_option: None,
//         str: "".to_string(),
//         normal_map: map,
//         path_map: map2,
//         vec: vec![],
//         path: PathBuf::from("C:\\msys64"),
//         nested_option: Some(NestedStruct { x: 44 }),
//         test_enum: HelloEnum::Struct(NestedStruct { x: 88 }),
//         test_enum_option: None,
//         wrapper: Arc::new("efg".to_string()),
//         str_option: None,
//         path_option: Some(PathBuf::from("C:\\msys64")),
//         vec_nested: vec![],
//         nested: NestedStruct { x: 400 },
//         tstruct: TStruct {
//             t: NestedStruct { x: 34 },
//         },
//         tstruct_option: Some(TStruct {
//             t: NestedStruct { x: 34 },
//         }),
//     };
//
//     let init2 = init.clone();
//
//     let x = ron::to_string(&init).unwrap();
//     println!("{}", &x);
//     let y: Hello = ron::from_str(&x).unwrap();
//     dbg!(&y);
//     assert_eq!(&init, &y);
//
//     #[derive(Debug, Config, PartialEq, Clone)]
//     struct TStruct<T> {
//         #[config(ty = "struct")]
//         t: T,
//     }
//
//     #[derive(Config)]
//     struct Hello2 {
//         #[config(ty = "struct")]
//         tstruct: TStruct<NestedStruct>,
//         #[config(ty = "_<struct>")]
//         tstruct2: Option<TStruct<NestedStruct>>,
//     }
// }
