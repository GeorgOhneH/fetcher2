// #![allow(dead_code)]
// #![feature(try_trait_v2)]
// #![feature(control_flow_enum)]
// #![feature(backtrace)]
// #![allow(unused_imports)]
//
// mod error;
// mod session;
// mod settings;
// mod site_modules;
// mod task;
// mod template;
//
// pub use error::{Result, TError};
//
// use crate::settings::DownloadSettings;
// use crate::template::{differ, DownloadArgs, Extensions, Mode, Template};
use config::{CBool, CInteger, CKwarg, CPath, CString, CType, Config};
use config_derive::Config;
// use futures::StreamExt;
// use log::{debug, error, info, log_enabled, Level};
use serde::Serialize;
// use std::collections::{HashMap, HashSet};
// use std::error::Error;
// use std::io;
// use std::ops::{Deref, DerefMut};
// use std::path::{Path, PathBuf};
// use std::sync::{Arc, Mutex, RwLock};
// use std::time::Instant;
// use tokio::sync::mpsc::{Receiver, Sender};
//
// fn main2() {
//     let mut base_config = fern::Dispatch::new();
//
//     base_config = base_config.level(log::LevelFilter::Trace);
//
//     let stdout_config = fern::Dispatch::new()
//         .filter(|metadata| {
//             !(metadata.target().starts_with("html5ever")
//                 || !metadata.target().starts_with("mio")
//                 || !metadata.target().starts_with("reqwest::connect"))
//         })
//         .format(|out, message, record| {
//             out.finish(format_args!(
//                 "[{}][{}][{}] {}",
//                 chrono::Local::now().format("%H:%M"),
//                 record.target(),
//                 record.level(),
//                 message
//             ))
//         })
//         .chain(io::stdout());
//
//     base_config.chain(stdout_config).apply().unwrap();
//
//     let mut template = Template::new();
//     tokio::runtime::Builder::new_multi_thread()
//         .enable_all()
//         .build()
//         .unwrap()
//         .block_on(async {
//             let session = crate::session::Session::new();
//             let dsettings = DownloadSettings {
//                 username: std::env::var("USERNAME").unwrap(),
//                 password: std::env::var("PASSWORD").unwrap(),
//                 save_path: PathBuf::from("C:\\programming\\rust\\fetcher2\\test"),
//                 download_args: DownloadArgs {
//                     extensions: Extensions {
//                         inner: HashSet::new(),
//                         mode: Mode::Forbidden,
//                     },
//                     keep_old_files: true,
//                 },
//                 force: false,
//             };
//             let start = Instant::now();
//             match template.run_root(session, dsettings).await {
//                 Ok(()) => {}
//                 Err(err) => {
//                     print!("{:?}", err);
//                     println!("{}", err.backtrace().unwrap());
//                     return;
//                 }
//             };
//             println!("{:#?}", start.elapsed());
//             let save_path = PathBuf::from("C:\\programming\\rust\\fetcher2\\test.yml");
//             template.save(&save_path).await.unwrap();
//             template.load(&save_path).await.unwrap();
//         });
// }
//
use config::ConfigEnum;
#[derive(Config, Serialize, Clone, Debug)]
struct Test {
    // #[config(default = true)]
    // hello: bool,
    // #[config(default = "dshf")]
    // hello2: String,
    // #[config(default = 0)]
    // int: isize,
    // path: Option<PathBuf>,
    #[config(ty = "enum")]
    efsdfs: Test2,
}

#[derive(Config, Serialize, Clone, Debug)]
enum Test2 {
    Hellfffffffffffffffo(String),
    Foo(String),
    Bar,
}

use druid::im::{vector, Vector};
use druid::lens::{self, LensExt};
use druid::text::{Formatter, ParseFormatter, Selection, Validation, ValidationError};
use druid::widget::{
    Button, CrossAxisAlignment, Flex, Label, LineBreaking, List, Scroll, Switch, TextBox,
};
use druid::{
    AppLauncher, Color, Data, Lens, LocalizedString, UnitPoint, Widget, WidgetExt, WindowDesc,
};
use std::collections::HashMap;
use std::path::PathBuf;
use config::CStruct;

#[derive(Clone, Data, Lens, Debug)]
struct AppData {
    cstruct: CStruct,
}

pub fn main() {
    let main_window = WindowDesc::new(ui_builder())
        .title(LocalizedString::new("list-demo-window-title").with_placeholder("List Demo"));
    let mut cstruct = Test::build_app();
    // let mut test: Test = Test::parse_from_app(&cstruct).unwrap();
    // test.efsdfs = Some(Test2::Bar);
    // test.update_app(&mut cstruct).unwrap();
    let data = AppData { cstruct };
    AppLauncher::with_window(main_window)
        .log_to_console()
        .launch(data)
        .expect("launch failed");
}

fn ui_builder() -> impl Widget<AppData> {
    let mut root = Flex::column();

    let mut lists = Flex::row().cross_axis_alignment(CrossAxisAlignment::Start);

    // Build a simple list
    lists.add_flex_child(
        Scroll::new(CStruct::widget())
            .vertical()
            .lens(AppData::cstruct),
        1.0,
    );

    root.add_child(
        Label::dynamic(|data, _env| format!("{:?}", data))
            .with_line_break_mode(LineBreaking::WordWrap),
    );
    root.add_flex_child(lists, 1.0);

    root.with_child(Label::new("horizontal list"))
        // .debug_paint_layout()
}