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
use crate::widgets::tree::NodeIndex;

selectors! {
    NEW_TEMPLATE: SingleUse<TemplateData>,
    NEW_EDIT_TEMPLATE: SingleUse<TemplateEditData>,
}

enum RunType {
    Root,
    Indexes(HashSet<NodeIndex>),
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

    sink.submit_command(
        NEW_TEMPLATE,
        SingleUse::new(init_template.widget_data()),
        Target::Global,
    )
    .unwrap();
    sink.submit_command(
        NEW_EDIT_TEMPLATE,
        SingleUse::new(init_template.widget_edit_data()),
        Target::Global,
    )
    .unwrap();

    let template = tokio::sync::RwLock::new(init_template);

    let mut futs = FuturesUnordered::new();
    let mut abort_handles = Vec::new();

    let fut = prepare_template(&template, dsettings.clone());
    add_new_future(fut, &mut futs, &mut abort_handles);

    let mut time = Instant::now();
    loop {
        tokio::select! {
            Ok(msg) = rx.recv_async() => {
                println!("{:?}", msg);
                match msg {
                    Msg::StartAll => {
                        let fut = run_template(&template, dsettings.clone(), RunType::Root);
                        add_new_future(fut, &mut futs, &mut abort_handles);
                    },
                    Msg::StartByIndex(indexes) => {
                        let fut = run_template(&template, dsettings.clone(), RunType::Indexes(indexes));
                        add_new_future(fut, &mut futs, &mut abort_handles);
                    },
                    Msg::Cancel => {
                        cancel_all(&mut abort_handles)
                    },
                    Msg::NewSettings(new_settings) => {
                        dsettings = Arc::new(new_settings);
                    },
                    Msg::NewTemplate(new_template) => {
                        cancel_all(&mut abort_handles);
                        sink.submit_command(NEW_TEMPLATE, SingleUse::new(new_template.widget_data()), Target::Global).unwrap();
                        sink.submit_command(NEW_EDIT_TEMPLATE, SingleUse::new(new_template.widget_edit_data()), Target::Global).unwrap();

                        let fut = replace_template(&template, new_template, dsettings.clone());
                        add_new_future(fut, &mut futs, &mut abort_handles);
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

fn add_new_future<'a>(
    future: impl Future<Output = ()> + Send + 'a,
    futs: &mut FuturesUnordered<BoxFuture<'a, std::result::Result<(), Aborted>>>,
    abort_handles: &mut Vec<AbortHandle>,
) {
    let (abort_handle, abort_registration) = AbortHandle::new_pair();
    let future = Abortable::new(future, abort_registration).boxed();
    abort_handles.push(abort_handle);
    futs.push(future);
}

fn cancel_all(abort_handles: &mut Vec<AbortHandle>) {
    for handle in abort_handles.iter() {
        handle.abort();
    }
    abort_handles.clear();
}

async fn replace_template(
    old_template: &tokio::sync::RwLock<Template>,
    new_template: Template,
    dsettings: Arc<DownloadSettings>,
) {
    let mut wl = old_template.write().await;
    wl.save().await.unwrap(); // TODO not panic
    *wl = new_template;
    wl.prepare(dsettings).await;
}

async fn run_template(
    template: &tokio::sync::RwLock<Template>,
    dsettings: Arc<DownloadSettings>,
    ty: RunType,
) {
    loop {
        let rl = template.read().await;
        if rl.is_prepared() {
            match ty {
                RunType::Root => rl.run_root(dsettings.clone()).await,
                RunType::Indexes(indexes) => rl.run(dsettings.clone(), &indexes).await,
            };
            return;
        } else {
            drop(rl);
            let mut wl = template.write().await;
            match wl.prepare(dsettings.clone()).await {
                Status::Success => {}
                Status::Failure => {
                    return;
                }
            }
        }
    }
}
async fn prepare_template(
    template: &tokio::sync::RwLock<Template>,
    dsettings: Arc<DownloadSettings>,
) {
    let mut wl = template.write().await;
    wl.prepare(dsettings.clone()).await;
}
