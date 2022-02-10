#![allow(dead_code)]
#![feature(try_trait_v2)]
#![feature(control_flow_enum)]
#![feature(backtrace)]
#![feature(type_alias_impl_trait)]
#![allow(clippy::new_without_default)]

use std::collections::HashMap;
use std::path::Path;
use std::path::PathBuf;
use std::time::{Duration, Instant};
use std::{fs, thread};

use config::ctypes::integer::RangedInt;
use config::ctypes::path::{Absolute, AbsoluteExistFile, AnyPath, StrictPath};
use config::deserializer::ConfigDeserializer;
use config::serializer::ConfigSerializer;
use config::traveller::{ConfigTraveller, Travel, Traveller, TravellerEnum};
use druid::{AppLauncher, LifeCycle, LocalizedString, WidgetExt, WindowDesc};
use lazy_static::lazy_static;
use self_update::cargo_crate_version;
use serde::{Deserialize, Serialize};

// use fetcher2::{Result, TError};

// use crate::background_thread::background_main;
// use crate::controller::{MainController, Msg};
// use crate::data::AppData;
// use crate::data::win::WindowState;
// use crate::ui::{build_ui, make_menu};

// mod background_thread;
// pub mod controller;
// mod cstruct_window;
// pub mod data;
// pub mod edit_window;
// pub mod ui;
// mod utils;
// pub mod widgets;
// pub mod communication;
//
// lazy_static! {
//     pub static ref CONFIG_DIR: PathBuf =
//         directories::ProjectDirs::from("ch", "fetcher2", "fetcher2")
//             .expect("Could not find a place to store the config files")
//             .config_dir()
//             .to_owned();
//     pub static ref WINDOW_STATE_DIR: PathBuf = Path::join(CONFIG_DIR.as_path(), "window_state.ron");
// }

// fn load_window_state() -> Result<AppData> {
//     let file_content = &fs::read(WINDOW_STATE_DIR.as_path())?;
//     Ok(ron::de::from_bytes::<AppData>(file_content)?)
// }
//
// fn build_window(
//     load_err: Option<TError>,
//     tx: flume::Sender<Msg>,
//     win_state: &Option<WindowState>,
// ) -> WindowDesc<AppData> {
//     let main_window = WindowDesc::new(
//         build_ui()
//             .controller(MainController::new(load_err, tx))
//             .padding(0.),
//     )
//     .menu(make_menu)
//     .title(LocalizedString::new("list-demo-window-title").with_placeholder("List Demo"));
//     if let Some(win_state) = win_state {
//         main_window
//             .window_size(win_state.get_size())
//             .set_position(win_state.get_pos())
//     } else {
//         main_window
//     }
// }
//
// pub fn main() {
//     let update_thread = thread::spawn(|| {
//         let status = self_update::backends::github::Update::configure()
//             .repo_owner("GeorgOhneH")
//             .repo_name("fetcher2")
//             .bin_name("github")
//             .show_download_progress(true)
//             .no_confirm(true)
//             .current_version(cargo_crate_version!())
//             .build()
//             .unwrap()
//             .update();
//         println!("Update status: `{:?}`!", status);
//     });
//
//     let (tx, rx) = flume::unbounded();
//     let (s, r) = crossbeam_channel::bounded(5);
//     let handle = thread::spawn(move || {
//         background_main(rx, r);
//     });
//
//     AppData::default().expect("AppData should always have a default");
//
//     let (app_data, load_err) = match load_window_state() {
//         Ok(data) => (data, None),
//         Err(err) => (
//             AppData::default().expect("AppData should always have a default"),
//             Some(err),
//         ),
//     };
//     let main_window = build_window(load_err, tx.clone(), &app_data.main_window);
//
//     let app_launcher = AppLauncher::with_window(main_window);
//
//     let sink = app_launcher.get_external_handle();
//     s.send(sink).unwrap();
//
//     use tracing_subscriber::prelude::*;
//     let filter_layer = tracing_subscriber::filter::LevelFilter::TRACE;
//     let filter = tracing_subscriber::filter::EnvFilter::default()
//         .add_directive("my_crate=trace".parse().unwrap())
//         .add_directive("druid=debug".parse().unwrap())
//         .add_directive("druid_widget_nursery=debug".parse().unwrap());
//     let fmt_layer = tracing_subscriber::fmt::layer()
//         // Display target (eg "my_crate::some_mod::submod") with logs
//         .with_target(true);
//
//     tracing_subscriber::registry()
//         .with(filter_layer)
//         .with(fmt_layer)
//         .with(filter)
//         .init();
//
//     app_launcher
//         // .log_to_console()
//         .launch(app_data)
//         .expect("launch failed");
//
//     let _ = tx.send(Msg::ExitAndSave);
//     update_thread.join().expect("thread panicked");
//     handle.join().expect("thread panicked");
// }

#[derive(Serialize, Deserialize, Debug, Travel)]
struct TestStruct2(#[travel(default = 9)] pub i64);

#[derive(Serialize, Deserialize, Debug, Travel)]
struct TestStruct3(pub i64, pub i64);

#[derive(Serialize, Deserialize, Debug, Travel)]
struct TestStruct4;

#[derive(Serialize, Deserialize, Debug, Travel)]
struct TestStruct {
    #[travel(default = 9)]
    pub field1: i64,
    pub field2: bool,
    pub field3: Option<bool>,
    pub field4: TestEnum,
    pub field5: (i64, i64),
    pub field6: [i64; 3],
    pub field7: Vec<i64>,
    pub field8: HashMap<String, i64>,
    pub field9: HashMap<PathBuf, i64>,
    pub field10: String,
    pub field11: PathBuf,
    pub field12: RangedInt<-10, 2>,
    pub field13: StrictPath<AnyPath>,
    pub field14: u64,
    pub field15: TestStruct2,
    pub field16: TestStruct3,
    pub field17: TestStruct4,
    pub field18: (),
}

#[derive(Serialize, Deserialize, Debug, Travel)]
enum TestEnum {
    Unit,
    One(i64),
    Two(i64, i64),
    Three { field0: i64, field1: i64 },
}

pub fn main() {
    let mut x = TestStruct::travel(&mut ConfigTraveller::new()).unwrap();
    let s = TestStruct {
        field1: 10,
        field2: true,
        field3: None,
        field4: TestEnum::Three {
            field0: 1,
            field1: 2,
        },
        field5: (1, 2),
        field6: [0; 3],
        field7: vec![9, 3, 6],
        field8: HashMap::from([("hello".to_string(), 10)]),
        field9: HashMap::from([(PathBuf::from("hello2"), 1), (PathBuf::from("he2"), 6)]),
        field10: String::from("hidushfi"),
        field11: PathBuf::from("hidushfi"),
        field12: RangedInt(0),
        field13: StrictPath::from("hello.yml"),
        field14: 19,
        field15: TestStruct2(10),
        field16: TestStruct3(10, 11),
        field17: TestStruct4,
        field18: (),
    };
    let t = Instant::now();
    s.serialize(&mut ConfigSerializer::new(&mut x)).unwrap();

    dbg!(t.elapsed());
    let r = TestStruct::deserialize(&mut ConfigDeserializer::new(&x)).unwrap();
    dbg!(t.elapsed());
    dbg!(r);
}
