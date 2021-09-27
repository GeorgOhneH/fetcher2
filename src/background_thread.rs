use std::any::Any;
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
use std::{io, thread};

use config::CStruct;
use config::{CBool, CInteger, CKwarg, CPath, CString, CType, Config};
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
use druid_widget_nursery::selectors;
use druid_widget_nursery::Tree;
use flume;
use futures::future::BoxFuture;
use futures::future::{AbortHandle, Abortable, Aborted};
use futures::prelude::stream::FuturesUnordered;
use futures::stream::FuturesOrdered;
use futures::{FutureExt, StreamExt};
use log::{debug, error, info, log_enabled, Level};
use serde::Serialize;
use tokio::time;
use tokio::time::Duration;

use crate::controller::Msg;
use crate::data::settings::DownloadSettings;
use crate::error::{Result, TError};
use crate::template::communication::RawCommunication;
use crate::template::nodes::node::Status;
use crate::template::widget_data::TemplateData;
use crate::template::widget_edit_data::TemplateEditData;
use crate::template::{DownloadArgs, Extensions, Mode, Template};
use crate::widgets::tree::NodeIndex;

selectors! {
    NEW_TEMPLATE: SingleUse<TemplateData>,
    NEW_EDIT_TEMPLATE: SingleUse<TemplateEditData>,

    MSG_FROM_THREAD: SingleUse<ThreadMsg>,
}

pub enum ThreadMsg {
    SettingsRequired,
    TemplateLoadingError(TError),
    TemplateSaveError(TError),
}

enum RunType {
    Root,
    Indexes(HashSet<NodeIndex>),
}

enum PostCommand {
    None,
    RunPrepare,
}

pub fn background_main(rx: flume::Receiver<Msg>, r: crossbeam_channel::Receiver<ExtEventSink>) {
    let sink = r.recv().expect("Should always work");
    tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .unwrap()
        .block_on(manager(rx, sink));
}

async fn manager(rx: flume::Receiver<Msg>, sink: ExtEventSink) {
    let template = tokio::sync::RwLock::new(Template::new());
    let mut dsettings: Option<Arc<DownloadSettings>> = None;

    let mut futs = FuturesUnordered::new();
    let mut abort_handles = Vec::new();

    loop {
        tokio::select! {
            Ok(msg) = rx.recv_async() => {
                println!("{:?}", msg);
                match msg {
                    Msg::StartAll => {
                        with_settings(
                            |settings| run_template(&template, settings, RunType::Root),
                            dsettings.clone(),
                            &mut futs,
                            &mut abort_handles,
                            sink.clone()
                        );
                    },
                    Msg::StartByIndex(indexes) => {
                        with_settings(
                            |settings| run_template(&template, settings, RunType::Indexes(indexes)),
                            dsettings.clone(),
                            &mut futs,
                            &mut abort_handles,
                            sink.clone()
                        );
                    },
                    Msg::Cancel => {
                        cancel_all(&mut abort_handles);
                        let fut = async { template.read().await.inform_of_cancel(); PostCommand::None };
                        add_new_future(fut, &mut futs, &mut abort_handles);
                    },
                    Msg::NewSettings(new_settings) => {
                        dsettings = Some(Arc::new(new_settings));
                        with_settings(
                            |settings| prepare_template(&template, settings),
                            dsettings.clone(),
                            &mut futs,
                            &mut abort_handles,
                            sink.clone()
                        );
                    },
                    Msg::NewTemplate(new_template) => {
                        cancel_all(&mut abort_handles);
                        let fut = replace_template(&template, new_template, sink.clone());
                        add_new_future(fut, &mut futs, &mut abort_handles);
                    },
                    Msg::NewTemplateByPath(path) => {
                        cancel_all(&mut abort_handles);
                        let fut = replace_template_by_path(&template, path, sink.clone());
                        add_new_future(fut, &mut futs, &mut abort_handles);
                    },
                    Msg::ExitAndSave => {
                        cancel_all(&mut abort_handles);
                        break;
                    }
                }
            }
            Some(result) = futs.next() => {
                if let Ok(cmd) = result {
                    handle_post_cmd(
                        cmd,
                        &template,
                        dsettings.clone(),
                        &mut futs,
                        &mut abort_handles,
                        sink.clone(),
                    )
                }
            },
            else => {
                println!("BREAK LOOP");
                break
            },
        }
    }

    while let Some(x) = futs.next().await {}
    // We use write so we are sure the other operations are finished
    template.write().await.save().await.unwrap();

    println!("GRACEFUL EXIT");
}

fn handle_post_cmd<'a>(
    cmd: PostCommand,
    template: &'a tokio::sync::RwLock<Template>,
    dsettings: Option<Arc<DownloadSettings>>,
    mut futs: &mut FuturesUnordered<BoxFuture<'a, std::result::Result<PostCommand, Aborted>>>,
    mut abort_handles: &mut Vec<AbortHandle>,
    sink: ExtEventSink,
) {
    match cmd {
        PostCommand::None => (),
        PostCommand::RunPrepare => {
            with_settings(
                |settings| prepare_template(template, settings),
                dsettings.clone(),
                &mut futs,
                &mut abort_handles,
                sink.clone(),
            );
        }
    }
}

fn with_settings<'a, T: Future<Output = PostCommand> + Send + 'a>(
    fut: impl FnOnce(Arc<DownloadSettings>) -> T,
    dsettings: Option<Arc<DownloadSettings>>,
    futs: &mut FuturesUnordered<BoxFuture<'a, std::result::Result<PostCommand, Aborted>>>,
    abort_handles: &mut Vec<AbortHandle>,
    sink: ExtEventSink,
) {
    if let Some(dsettings) = dsettings {
        add_new_future(fut(dsettings.clone()), futs, abort_handles);
    } else {
        sink.submit_command(
            MSG_FROM_THREAD,
            SingleUse::new(ThreadMsg::SettingsRequired),
            Target::Global,
        )
        .unwrap()
    }
}

fn add_new_future<'a>(
    future: impl Future<Output = PostCommand> + Send + 'a,
    futs: &mut FuturesUnordered<BoxFuture<'a, std::result::Result<PostCommand, Aborted>>>,
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

async fn replace_template_by_path(
    old_template: &tokio::sync::RwLock<Template>,
    path: PathBuf,
    sink: ExtEventSink,
) -> PostCommand {
    let comm = RawCommunication::new(sink.clone());
    dbg!("starting load");
    match Template::load(path.as_path(), comm).await {
        Ok(new_template) => replace_template(old_template, new_template, sink).await,
        Err(err) => {
            sink.submit_command(
                MSG_FROM_THREAD,
                SingleUse::new(ThreadMsg::TemplateLoadingError(err)),
                Target::Global,
            )
            .unwrap();
            PostCommand::None
        }
    }
}

async fn replace_template(
    old_template: &tokio::sync::RwLock<Template>,
    new_template: Template,
    sink: ExtEventSink,
) -> PostCommand {
    dbg!("replace load");
    old_template.read().await.inform_of_cancel();
    let mut wl = old_template.write().await;
    sink.submit_command(
        NEW_TEMPLATE,
        SingleUse::new(new_template.widget_data()),
        Target::Global,
    )
    .unwrap();
    sink.submit_command(
        NEW_EDIT_TEMPLATE,
        SingleUse::new(new_template.widget_edit_data()),
        Target::Global,
    )
    .unwrap();
    if let Err(err) = wl.save().await {
        sink.submit_command(
            MSG_FROM_THREAD,
            SingleUse::new(ThreadMsg::TemplateSaveError(err)),
            Target::Global,
        )
        .unwrap()
    }
    if let Err(err) = new_template.save().await {
        sink.submit_command(
            MSG_FROM_THREAD,
            SingleUse::new(ThreadMsg::TemplateSaveError(err)),
            Target::Global,
        )
        .unwrap()
    }
    *wl = new_template;
    dbg!("finished");
    PostCommand::RunPrepare
}

async fn run_template(
    template: &tokio::sync::RwLock<Template>,
    dsettings: Arc<DownloadSettings>,
    ty: RunType,
) -> PostCommand {
    loop {
        let rl = template.read().await;
        if rl.is_prepared() {
            match ty {
                RunType::Root => rl.run_root(dsettings.clone()).await,
                RunType::Indexes(indexes) => rl.run(dsettings.clone(), &indexes).await,
            };
            return PostCommand::None;
        } else {
            drop(rl);
            let mut wl = template.write().await;
            match wl.prepare(dsettings.clone()).await {
                Status::Success => {}
                Status::Failure => {
                    return PostCommand::None;
                }
            }
        }
    }
}
async fn prepare_template(
    template: &tokio::sync::RwLock<Template>,
    dsettings: Arc<DownloadSettings>,
) -> PostCommand {
    dbg!("start prepare");
    let mut wl = template.write().await;
    wl.prepare(dsettings.clone()).await;
    PostCommand::None
}
