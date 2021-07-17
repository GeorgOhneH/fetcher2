use crate::delegate::{Msg, TemplateDelegate};
use crate::template::widget::{TemplateData, TemplateWidget};
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
use crate::template::{DownloadArgs, Extensions, Mode, Template};
use config::{CBool, CInteger, CKwarg, CPath, CString, CType, Config};
use config_derive::Config;
use futures::future::{AbortHandle, Abortable, Aborted};
use futures::prelude::stream::FuturesUnordered;
use futures::stream::FuturesOrdered;
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

pub fn background_main(sink: ExtEventSink, rx: flume::Receiver<Msg>, template: Template) {
    tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .unwrap()
        .block_on(manager(sink, rx, template));
}

async fn manager(sink: ExtEventSink, rx: flume::Receiver<Msg>, mut template: Template) {
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

    match template.prepare(dsettings.clone()).await {
        Ok(()) => {}
        Err(err) => {
            print!("{:?}", err);
            println!("{}", err.backtrace().unwrap());
            return;
        }
    };

    let mut futs = FuturesUnordered::new();
    let mut abort_handles = Vec::new();
    loop {
        tokio::select! {
            Ok(msg) = rx.recv_async() => {
                println!("{:?}", msg);
                match msg {
                    Msg::StartAll => {
                        let (abort_handle, abort_registration) = AbortHandle::new_pair();
                        let future = Abortable::new(template.run_root(dsettings.clone()), abort_registration);
                        abort_handles.push(abort_handle);
                        futs.push(future);
                        println!("{:?}, {:?}", abort_handles.len(), futs.len())
                    },
                    Msg::Cancel => {
                        for handle in abort_handles.iter() {
                            handle.abort();
                        }
                        abort_handles.clear();
                    },
                }
            }
            Some(result) = futs.next() => {
                println!("finished future {:?}", result);
                match result {
                    Ok(Ok(())) => {}
                    Ok(Err(err)) => {
                        print!("{:?}", err);
                        println!("{}", err.backtrace().unwrap());
                        return;
                    },
                    Err(err) => {},
                }
            },
            else => {println!("BREAK LOOP");break},
        }
    }
}
