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
use config_derive::Config;
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

//
pub fn main() {
    let win_state = if let Ok(file_content) = &fs::read(WINDOW_STATE_DIR.as_path()) {
        let file_str = String::from_utf8_lossy(file_content);
        if let Ok(win_state) = serde_json::from_str::<MainWindowState>(&file_str) {
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
