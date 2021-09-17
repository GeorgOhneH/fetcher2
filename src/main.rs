// #![allow(dead_code)]
// #![feature(try_trait_v2)]
// #![feature(control_flow_enum)]
// #![feature(backtrace)]
// #![feature(type_alias_impl_trait)]
// #![allow(unused_imports)]
//
// use std::any::Any;
// use std::cmp::max;
// use std::collections::HashMap;
// use std::collections::HashSet;
// use std::error::Error;
// use std::future::Future;
// use std::ops::{Deref, DerefMut};
// use std::path::Path;
// use std::path::PathBuf;
// use std::pin::Pin;
// use std::sync::{Arc, Mutex, RwLock};
// use std::time::Instant;
// use std::{fs, io, thread};
//
// use config::CStruct;
// use config::State;
// use config::{CBool, CInteger, CKwarg, CPath, CString, CType, Config};
// use druid::im::{vector, Vector};
// use druid::lens::{self, InArc, LensExt};
// use druid::text::{Formatter, ParseFormatter, Selection, Validation, ValidationError};
// use druid::widget::{
//     Button, Checkbox, Controller, CrossAxisAlignment, Either, Flex, Label, LineBreaking, List,
//     Maybe, Scroll, SizedBox, Spinner, Switch, TextBox,
// };
// use druid::{
//     im, AppDelegate, AppLauncher, Application, Color, Command, Data, DelegateCtx, Env, Event,
//     EventCtx, ExtEventSink, Handled, LayoutCtx, Lens, LifeCycle, LifeCycleCtx, LocalizedString,
//     Menu, MenuItem, MouseButton, PaintCtx, Point, Screen, Selector, SingleUse, Size, Target,
//     UnitPoint, UpdateCtx, Vec2, Widget, WidgetExt, WidgetId, WidgetPod, WindowConfig, WindowDesc,
//     WindowLevel,
// };
// use flume;
// use futures::future::BoxFuture;
// use futures::StreamExt;
// use log::{debug, error, info, log_enabled, Level};
// use serde::Serialize;
// use tokio::time;
// use tokio::time::Duration;
//
// pub use error::{Result, TError};
//
// use crate::background_thread::background_main;
// use crate::controller::{AppState, MainController, Msg, WINDOW_STATE_DIR};
// use crate::cstruct_window::CStructBuffer;
// use crate::settings::{DownloadSettings, Settings, Test};
// use crate::template::communication::RawCommunication;
// use crate::template::nodes::node_data::NodeData;
// use crate::template::nodes::root_data::RootNodeData;
// use crate::template::widget_data::TemplateData;
// use crate::template::{DownloadArgs, Extensions, Mode, Template};
// use crate::ui::{build_ui, make_menu, AppData, TemplateInfoSelect};
// use crate::widgets::file_watcher::FileWatcher;
// use crate::widgets::header::Header;
// use crate::widgets::tree::Tree;
// use std::io::Write;
//
// mod background_thread;
// pub mod controller;
// mod cstruct_window;
// pub mod edit_window;
// mod error;
// mod session;
// mod settings;
// mod site_modules;
// mod task;
// mod template;
// pub mod ui;
// mod utils;
// pub mod widgets;
//
// fn load_window_state() -> Result<AppState> {
//     let file_content = &fs::read(WINDOW_STATE_DIR.as_path())?;
//     Ok(ron::de::from_bytes::<AppState>(file_content)?)
// }
// fn save_window_state(app_state: &AppState) -> Result<()> {
//     let serialized = ron::to_string(app_state)?;
//
//     fs::create_dir_all(WINDOW_STATE_DIR.as_path().parent().expect(""))?;
//
//     let mut f = fs::OpenOptions::new()
//         .write(true)
//         .truncate(true)
//         .create(true)
//         .open(WINDOW_STATE_DIR.as_path())?;
//     f.write_all(&serialized.as_bytes())?;
//     Ok(())
// }
//
// fn build_window(
//     app_state: Arc<RwLock<AppState>>,
//     load_err: Option<TError>,
//     tx: flume::Sender<Msg>,
// ) -> WindowDesc<AppData> {
//     let mut main_window = WindowDesc::new(build_ui().controller(MainController::new(
//         app_state.clone(),
//         load_err,
//         tx,
//     )))
//     .menu(make_menu)
//     .title(LocalizedString::new("list-demo-window-title").with_placeholder("List Demo"));
//     if let Some(win_state) = &app_state.read().unwrap().main_window {
//         main_window = main_window
//             .set_position(win_state.pos)
//             .window_size(win_state.size);
//     }
//     main_window
// }
//
// pub fn main() {
//     let (raw_app_state, load_err) = match load_window_state() {
//         Ok(win_state) => (win_state, None),
//         Err(err) => (AppState::default(), Some(err)),
//     };
//     let app_state = Arc::new(RwLock::new(raw_app_state));
//
//     let (tx, rx) = flume::unbounded();
//     let (s, r) = crossbeam_channel::bounded(5);
//     let handle = thread::spawn(move || {
//         background_main(rx, r);
//     });
//
//     let main_window = build_window(app_state.clone(), load_err, tx.clone());
//
//     let app_launcher = AppLauncher::with_window(main_window);
//
//     let sink = app_launcher.get_external_handle();
//     s.send(sink).unwrap();
//
//     let data = AppData {
//         template: TemplateData::new(),
//         settings: None,
//         recent_templates: Vector::new(),
//         template_info_select: TemplateInfoSelect::Nothing,
//     };
//
//     use tracing_subscriber::prelude::*;
//     let filter_layer = tracing_subscriber::filter::LevelFilter::DEBUG;
//     let filter = tracing_subscriber::filter::EnvFilter::default()
//         .add_directive("my_crate=trace".parse().unwrap())
//         .add_directive("druid=trace".parse().unwrap())
//         .add_directive("druid_widget_nursery=trace".parse().unwrap());
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
//         .launch(data)
//         .expect("launch failed");
//
//     tx.send(Msg::ExitAndSave).expect("");
//     let _ = save_window_state(&app_state.read().unwrap());
//     handle.join().expect("thread panicked");
// }


use config::{CStruct, Config, ConfigEnum};
use ron::ser::PrettyConfig;
use ron::{Map, Value};
use serde::{Serialize};
use std::path::PathBuf;
use std::sync::{Arc, Mutex, RwLock};
use std::collections::HashMap;
use serde::__private::fmt::Debug;
use std::marker::PhantomData;

//
fn main() {
    #[derive(Debug, Config, PartialEq, Clone)]
    struct Hello {
        bool: bool,
        bool_option: Option<bool>,
        int: isize,
        int_option: Option<isize>,
        str: String,
        str_option: Option<String>,
        path: PathBuf,
        path_option: Option<PathBuf>,
        normal_map: HashMap<String, Vec<String>>,
        #[config(ty = "_<_, struct>")]
        path_map: HashMap<PathBuf, NestedStruct>,

        vec: Vec<isize>,
        #[config(ty = "_<struct>")]
        vec_nested: Vec<NestedStruct>,

        #[config(ty = "struct")]
        nested: NestedStruct,
        #[config(ty = "_<struct>", skip = None)]
        nested_option: Option<NestedStruct>,

        #[config(ty = "enum")]
        test_enum: HelloEnum,

        #[config(ty = "_<enum>")]
        test_enum_option: Option<HelloEnum>,

        wrapper: Arc<String>,
    }

    #[derive(Debug, ConfigEnum, PartialEq, Clone)]
    enum HelloEnum {
        Unit,
        #[config(ty = "struct")]
        Struct(NestedStruct),
        With(String),
    }

    #[derive(Debug, Config, PartialEq, Clone)]
    struct NestedStruct {
        x: isize,
    }

    let mut map = HashMap::new();
    map.insert("Hello".to_string(), vec!["hello again".to_string()]);
    let mut map2 = HashMap::new();
    map2.insert(PathBuf::from("Hello"), NestedStruct { x: 42 });

    let init = Hello {
        bool: true,
        bool_option: Some(false),
        int: -10,
        int_option: None,
        str: "".to_string(),
        normal_map: map,
        path_map: map2,
        vec: vec![],
        path: PathBuf::from("C:\\msys64"),
        nested_option: Some(NestedStruct { x: 44 }),
        test_enum: HelloEnum::Struct(NestedStruct { x: 88 }),
        test_enum_option: None,
        wrapper: Arc::new("efg".to_string()),
        str_option: None,
        path_option: Some(PathBuf::from("C:\\msys64")),
        vec_nested: vec![],
        nested: NestedStruct { x: 400 }
    };

    let init2 = init.clone();

    let x = ron::to_string(&init).unwrap();
    println!("{}", &x);
    let y: Hello = ron::from_str(&x).unwrap();
    dbg!(&y);
    assert_eq!(&init, &y);

    #[derive(Config, Debug)]
    pub struct SubWindowInfo<T> {
        #[config(ty = "struct")]
        data_state: T,
    }

    impl<T: Config> ::config::Config for SubWindowInfo<T> {
        fn builder() -> ::config::CStructBuilder {
            ::config::CStructBuilder::new().arg(
                ::config::CKwargBuilder::new(
                    "data_state".to_string(),
                    ::config::CType::CStruct(T::builder().name("data_state".to_string()).build()),
                )
                    .required(true)
                    .build(),
            )
        }
        fn parse_from_app(
            app: &::config::CStruct,
        ) -> std::result::Result<Self, ::config::RequiredError> {
            let data_state = match {
                match app.get_ty(&"data_state".to_string()).unwrap() {
                    ::config::CType::CStruct(config_struct) => {
                        match T::parse_from_app(config_struct) {
                            Ok(value) => Ok(Some(value)),
                            Err(err) => Err(err),
                        }
                    }
                    _ => ::core::panicking::panic_fmt(
                        match match () {
                            () => [],
                        } {
                            ref args => unsafe {
                                ::core::fmt::Arguments::new_v1(&["This should never happen"], args)
                            },
                        },
                    ),
                }
            } {
                Ok(value) => match value {
                    Some(x) => Ok(x),
                    None => Err(::config::RequiredError::new(
                        "data_state",
                        "Must be Option?",
                    )),
                },
                Err(err) => Err(err),
            }?;
            Ok(Self { data_state })
        }
        fn update_app(
            self,
            app: &mut ::config::CStruct,
        ) -> std::result::Result<(), ::config::InvalidError> {
            let results: Vec<std::result::Result<(), ::config::InvalidError>> =
                <[_]>::into_vec(box [{
                    match app.get_ty_mut("data_state").unwrap() {
                        ::config::CType::CStruct(ref mut config_struct) => {
                            T::update_app(self.data_state, config_struct)
                        }
                        _ => ::core::panicking::panic_fmt(
                            match match () {
                                () => [],
                            } {
                                ref args => unsafe {
                                    ::core::fmt::Arguments::new_v1(
                                        &["This should never happen"],
                                        args,
                                    )
                                },
                            },
                        ),
                    }
                }]);
            for result in results {
                if let Err(err) = result {
                    return Err(err);
                }
            }
            Ok(())
        }
    }
    impl<T> serde::Serialize for SubWindowInfo<T> {
        fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
            where
                S: serde::Serializer,
        {
            use serde::ser::SerializeStruct as _;
            let mut state = serializer.serialize_struct("SubWindowInfo", 1usize)?;
            state.serialize_field("data_state", &self.data_state)?;
            state.end()
        }
    }
    impl<'de, T> serde::Deserialize<'de> for SubWindowInfo<T> {
        fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
            where
                D: serde::Deserializer<'de>,
        {
            const FIELDS: &'static [&'static str] = &["data_state"];
            #[allow(non_camel_case_types)]
            enum Field {
                data_state,
                __Nothing,
            }
            impl<'de> serde::Deserialize<'de> for Field {
                fn deserialize<D>(deserializer: D) -> std::result::Result<Field, D::Error>
                    where
                        D: serde::Deserializer<'de>,
                {
                    struct FieldVisitor;
                    impl<'de> serde::de::Visitor<'de> for FieldVisitor {
                        type Value = Field;
                        fn expecting(
                            &self,
                            formatter: &mut std::fmt::Formatter,
                        ) -> std::fmt::Result {
                            formatter.write_str("not valid field found")
                        }
                        fn visit_str<E>(self, value: &str) -> std::result::Result<Field, E>
                            where
                                E: serde::de::Error,
                        {
                            match value {
                                "data_state" => Ok(Field::data_state),
                                _ => Ok(Field::__Nothing),
                            }
                        }
                    }
                    deserializer.deserialize_identifier(FieldVisitor)
                }
            }
            struct DurationVisitor;
            impl<'de> serde::de::Visitor<'de> for DurationVisitor {
                type Value = SubWindowInfo;
                fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                    formatter.write_str("struct #name")
                }
                fn visit_map<V>(self, mut map: V) -> std::result::Result<Self::Value, V::Error>
                    where
                        V: serde::de::MapAccess<'de>,
                {
                    let mut cstruct: config::CStruct = SubWindowInfo::builder().build();
                    let mut data_state = None;
                    while let Ok(Some(key)) = map.next_key() {
                        match key {
                            Field::data_state => {
                                if data_state.is_some() {
                                    return Err(serde::de::Error::duplicate_field("data_state"));
                                }
                                if let Ok(value) = map.next_value() {
                                    data_state = Some(value);
                                }
                            }
                            Field::__Nothing => {
                                let _: std::result::Result<(), _> = map.next_value();
                            }
                        }
                    }
                    if let Some(value) = data_state {
                        let value_hint: T = value;
                        let _ = value_hint.update_app(
                            &mut cstruct
                                .get_ty_mut("data_state")
                                .unwrap()
                                .struct_mut()
                                .unwrap(),
                        );
                    }
                    SubWindowInfo::parse_from_app(&cstruct)
                        .map_err(|err| serde::de::Error::custom(err.msg))
                }
            }
            deserializer.deserialize_struct("SubWindowInfo", FIELDS, DurationVisitor)
        }
    }
    #[automatically_derived]
    #[allow(unused_qualifications)]
    impl<T: ::core::fmt::Debug> ::core::fmt::Debug for SubWindowInfo<T> {
        fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
            match *self {
                SubWindowInfo {
                    data_state: ref __self_0_0,
                } => {
                    let debug_trait_builder =
                        &mut ::core::fmt::Formatter::debug_struct(f, "SubWindowInfo");
                    let _ = ::core::fmt::DebugStruct::field(
                        debug_trait_builder,
                        "data_state",
                        &&(*__self_0_0),
                    );
                    ::core::fmt::DebugStruct::finish(debug_trait_builder)
                }
            }
        }
    }



}
