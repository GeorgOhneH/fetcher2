#![allow(dead_code)]
#![feature(try_trait_v2)]
#![feature(control_flow_enum)]
#![feature(backtrace)]
#![feature(type_alias_impl_trait)]
#![allow(unused_imports)]

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

pub use error::{Result, TError};

use crate::background_thread::background_main;
use crate::controller::{MainController, MainWindowState, WINDOW_STATE_DIR};
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
use tokio::time;
use tokio::time::Duration;

// //
pub fn main() {
    let win_state = if let Ok(file_content) = &fs::read(WINDOW_STATE_DIR.as_path()) {
        let file_str = String::from_utf8_lossy(file_content);
        if let Ok(win_state) = ron::from_str::<MainWindowState>(&file_str) {
            win_state
        } else {
            MainWindowState::default()
        }
    } else {
        MainWindowState::default()
    };

    let (tx, rx) = flume::unbounded();
    let (s, r) = crossbeam_channel::bounded(5);
    let handle = thread::spawn(move || {
        background_main(rx, r);
    });

    let pos = win_state.win_pos;
    let size = win_state.win_size;
    let mut main_window =
        WindowDesc::new(build_ui().controller(MainController::new(win_state, tx)))
            .menu(make_menu)
            .title(LocalizedString::new("list-demo-window-title").with_placeholder("List Demo"));
    if let Some(pos) = pos {
        main_window = main_window.set_position(pos);
    }
    if let Some(size) = size {
        main_window = main_window.window_size(size);
    }

    let app_launcher = AppLauncher::with_window(main_window);

    s.send(app_launcher.get_external_handle()).unwrap();

    let data = AppData {
        template: TemplateData::new(),
        settings: None,
        recent_templates: Vector::new(),
        template_info_select: TemplateInfoSelect::Nothing,
    };

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
        .launch(data)
        .expect("launch failed");

    handle.join().unwrap();
}

//
// use config::{CStruct, Config, ConfigEnum};
// use ron::ser::PrettyConfig;
// use ron::{Map, Value};
// use serde::{Serialize};
// use std::path::PathBuf;
// use std::sync::{Arc, Mutex, RwLock};
// use std::collections::HashMap;
//
// //
// fn main() {
//
//     #[derive(Serialize, Debug, Config, PartialEq, Clone)]
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
//         #[config(ty = "_<_, Struct>")]
//         path_map: HashMap<PathBuf, NestedStruct>,
//
//         vec: Vec<isize>,
//         #[config(ty = "_<Struct>")]
//         vec_nested: Vec<NestedStruct>,
//
//         #[config(ty = "Struct")]
//         nested: NestedStruct,
//         #[config(ty = "_<Struct>")]
//         nested_option: Option<NestedStruct>,
//
//         #[config(ty = "Enum")]
//         test_enum: HelloEnum,
//
//         #[config(ty = "_<Enum>")]
//         test_enum_option: Option<HelloEnum>,
//
//         wrapper: Arc<String>,
//     }
//
//
//
//     #[derive(Debug, Serialize, ConfigEnum, PartialEq, Clone)]
//     enum HelloEnum {
//         Unit,
//         #[config(ty = "Struct")]
//         Struct(NestedStruct),
//         With(String),
//     }
//
//
//     #[derive(Serialize, Debug, Config, PartialEq, Clone)]
//     struct NestedStruct {
//         x: isize,
//     }
//
//     let mut map = HashMap::new();
//     map.insert("Hello".to_string(), vec!["hello again".to_string()]);
//     let mut map2 = HashMap::new();
//     map2.insert(PathBuf::from("Hello"), NestedStruct {x: 42});
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
//         nested_option: None,
//         test_enum: HelloEnum::Struct(NestedStruct {x: 88}),
//         test_enum_option: None,
//         wrapper: Arc::new("efg".to_string()),
//         str_option: None,
//         path_option: Some(PathBuf::from("C:\\msys64")),
//         vec_nested: vec![],
//         nested: NestedStruct { x: 400 }
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
// }
