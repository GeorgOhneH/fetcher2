use crate::template::widget_data::{TemplateData};
use crate::template::{Template};
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
use std::collections::{HashMap, HashSet};
use std::future::Future;
use std::path::PathBuf;
use std::pin::Pin;
use tokio::time;
use tokio::time::Duration;
use std::thread;
use crate::background_thread::{background_main, EDIT_DATA};
use crate::widgets::tree::NodeIndex;
use crate::settings::DownloadSettings;
use druid_widget_nursery::selectors;
use crate::template::widget_edit_data::TemplateEditData;

#[derive(Debug)]
pub enum Msg {
    StartAll,
    StartByIndex(HashSet<NodeIndex>),
    Cancel,
    NewSettings(DownloadSettings),

    RequestEditData(WidgetId),
    UpdateEditData(TemplateEditData),
}

#[derive(Debug)]
pub enum TemplateMsg {
    StartAll,
    StartByIndex(HashSet<NodeIndex>),
    Cancel,
    NewSettings(DownloadSettings)
}

selectors! {
    MSG_THREAD: SingleUse<Msg>
}

pub struct TemplateDelegate {
    tx: flume::Sender<Msg>,
}

impl TemplateDelegate {
    pub fn new(sink: ExtEventSink, template: Template) -> Self {
        let (tx, rx) = flume::unbounded();
        thread::spawn(move || {
            background_main(sink, rx, template);
        });
        Self { tx }
    }
}

impl<T: Data> AppDelegate<T> for TemplateDelegate {
    fn command(
        &mut self,
        _ctx: &mut DelegateCtx,
        _target: Target,
        cmd: &Command,
        _data: &mut T,
        _env: &Env,
    ) -> Handled {
        if cmd.is(MSG_THREAD) {
            let msg = cmd.get_unchecked(MSG_THREAD).take().unwrap();
            self.tx.send(msg).unwrap();
            Handled::Yes
        } else {
            Handled::No
        }
    }
}
