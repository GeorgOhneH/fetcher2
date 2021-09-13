use crate::delegate::{Msg, TemplateDelegate};
use crate::template::widget_data::TemplateData;
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

use crate::error::{Result, TError};

use crate::settings::DownloadSettings;
use crate::template::nodes::node::Status;
use crate::template::{DownloadArgs, Extensions, Mode, Template};
use config::{CBool, CInteger, CKwarg, CPath, CString, CType, Config};
use config_derive::Config;
use druid_widget_nursery::selectors;
use futures::future::{AbortHandle, Abortable, Aborted};
use futures::prelude::stream::FuturesUnordered;
use futures::stream::FuturesOrdered;
use futures::{FutureExt, StreamExt};
use log::{debug, error, info, log_enabled, Level};
use serde::Serialize;
use std::collections::HashSet;
use std::error::Error;
use std::ops::{Deref, DerefMut};
use std::path::Path;
use std::sync::{Arc, Mutex, RwLock};
use std::time::Instant;
use std::{io, thread};

use crate::template::communication::RawCommunication;
use crate::template::widget_edit_data::TemplateEditData;

selectors! {
    EDIT_DATA: SingleUse<TemplateEditData>,
    NEW_TEMPLATE: SingleUse<TemplateData>,
}

pub fn background_main(sink: ExtEventSink, rx: flume::Receiver<Msg>, template: Template) {
    tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .unwrap()
        .block_on(manager(sink, rx, template));
}

async fn manager(sink: ExtEventSink, rx: flume::Receiver<Msg>, init_template: Template) {
    let mut dsettings = Arc::new(DownloadSettings {
        username: Some(std::env::var("USERNAME").unwrap()),
        password: Some(std::env::var("PASSWORD").unwrap()),
        save_path: PathBuf::from("C:\\programming\\rust\\fetcher2\\test"),
        download_args: DownloadArgs {
            extensions: Extensions {
                inner: im::HashSet::new(),
                mode: Mode::Forbidden,
            },
            keep_old_files: true,
        },
        x: Vector::new(),
        force: false,
    });

    let template = tokio::sync::RwLock::new(init_template);

    match template.write().await.prepare(dsettings.clone()).await {
        Status::Success => {}
        Status::Failure => {
            dbg!("Got error in prepare exit...");
            // TODO
            return;
        }
    };

    let mut futs = FuturesUnordered::new();
    let mut abort_handles = Vec::new();
    let mut time = Instant::now();
    loop {
        tokio::select! {
            Ok(msg) = rx.recv_async() => {
                println!("{:?}", msg);
                match msg {
                    Msg::StartAll => {
                        let (abort_handle, abort_registration) = AbortHandle::new_pair();
                        let dsetting_clone = dsettings.clone();
                        let future = Abortable::new(async {
                            let rl = template.read().await;
                            rl.run_root(dsetting_clone).await;
                        } , abort_registration).boxed();
                        abort_handles.push(abort_handle);
                        futs.push(future);
                        println!("{:?}, {:?}", abort_handles.len(), futs.len());
                        time = Instant::now();
                    },
                    Msg::StartByIndex(indexes) => {
                        let (abort_handle, abort_registration) = AbortHandle::new_pair();
                        let dsetting_clone = dsettings.clone();
                        let future = Abortable::new(async {
                            let rl = template.read().await;
                            rl.run(dsetting_clone, Some(indexes)).await;
                        } , abort_registration).boxed();
                        abort_handles.push(abort_handle);
                        futs.push(future);
                        println!("{:?}, {:?}", abort_handles.len(), futs.len());
                        time = Instant::now();
                    },
                    Msg::Cancel => {
                        for handle in abort_handles.iter() {
                            handle.abort();
                        }
                        abort_handles.clear();
                    },
                    Msg::NewSettings(new_settings) => {
                        dsettings = Arc::new(new_settings);
                    },
                    Msg::RequestEditData(widget_id) => {
                        let edit_data = template.read().await.widget_edit_data();
                        sink.submit_command(EDIT_DATA, SingleUse::new(edit_data), Target::Widget(widget_id)).unwrap();
                    },
                    Msg::UpdateEditData(edit_data) => {
                        for handle in abort_handles.iter() {
                            handle.abort();
                        }
                        abort_handles.clear();

                        let comm = RawCommunication::new(sink.clone());
                        let new_template = Template::from_raw(edit_data, comm);
                        sink.submit_command(NEW_TEMPLATE, SingleUse::new(new_template.widget_data()), Target::Global).unwrap();

                        let (abort_handle, abort_registration) = AbortHandle::new_pair();
                        let dsetting_clone = dsettings.clone();
                        let future = Abortable::new(async {
                            let mut wl = template.write().await;
                            *wl = new_template;
                            wl.prepare(dsetting_clone).await;
                        }, abort_registration).boxed();
                        abort_handles.push(abort_handle);
                        futs.push(future);
                    },
                }
            }
            Some(result) = futs.next() => {
                println!("finished future {:?}, time: {:?}", result, time.elapsed());
            },
            else => {
                println!("BREAK LOOP");
                break
            },
        }
    }
}
