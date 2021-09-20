use std::{fs, thread};
use std::any::Any;
use std::cmp::max;
use std::collections::HashSet;
use std::convert::TryFrom;
use std::fmt::{Debug, Display, Formatter};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex, RwLock};
use std::thread::JoinHandle;
use std::time::Duration;

use config::{Config, InvalidError, RequiredError};
use directories::{BaseDirs, ProjectDirs, UserDirs};
use druid::{
    Command, commands, ExtEventSink, HasRawWindowHandle, Menu, MenuItem, RawWindowHandle, Rect,
    Scalable, Selector, SingleUse, Target, theme, WidgetExt, WidgetId, WindowConfig, WindowHandle,
    WindowLevel,
};
use druid::{
    BoxConstraints, Data, Env, Event, EventCtx, LayoutCtx, Lens, LifeCycle, LifeCycleCtx, PaintCtx,
    Point, UpdateCtx, Widget, WidgetPod,
};
use druid::commands::{CLOSE_WINDOW, QUIT_APP};
use druid::im::Vector;
use druid::kurbo::{BezPath, Size};
use druid::piet::{LineCap, LineJoin, RenderContext, StrokeStyle};
use druid::widget::{Controller, Label};
use druid_widget_nursery::{selectors, Wedge};
use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};
use tokio::io::AsyncWriteExt;
use url::Position;

use crate::{Result, TError};
use crate::background_thread::{
    background_main, MSG_FROM_THREAD, NEW_EDIT_TEMPLATE, NEW_TEMPLATE, ThreadMsg,
};
use crate::controller::{Msg, MSG_THREAD};
use crate::cstruct_window::c_option_window;
use crate::data::AppData;
use crate::edit_window::edit_window;
use crate::template::communication::NODE_EVENT;
use crate::template::nodes::node::NodeEvent;
use crate::template::nodes::node_data::NodeData;
use crate::template::nodes::root_data::RootNodeData;
use crate::template::Template;
use crate::template::widget_data::TemplateData;
use crate::template::widget_edit_data::TemplateEditData;
use crate::utils::show_err;
use crate::widgets::sub_window_widget::SubWindow;
use crate::widgets::tree::{DataNodeIndex, NodeIndex, Tree};

pub struct TemplateController {}

impl TemplateController {
    pub fn new() -> Self {
        Self {}
    }

    fn new_template(data: &mut AppData, template_data: TemplateData) {
        if let Some(new_path) = &template_data.save_path {
            data.recent_templates.retain(|path| path != new_path);
            data.recent_templates.push_front(new_path.clone());
        }
        data.template = template_data;
    }
}

impl<W: Widget<AppData>> Controller<AppData, W> for TemplateController {
    fn event(
        &mut self,
        child: &mut W,
        ctx: &mut EventCtx,
        event: &Event,
        data: &mut AppData,
        env: &Env,
    ) {
        match event {
            Event::Command(cmd) if cmd.is(NODE_EVENT) => {
                ctx.set_handled();
                let (node_event, idx) = cmd.get_unchecked(NODE_EVENT).take().unwrap();
                let node = data.template.node_mut(&idx);
                node.update_node(node_event);
                return;
            }
            Event::Command(cmd) if cmd.is(NEW_TEMPLATE) => {
                ctx.set_handled();
                let template_data = cmd.get_unchecked(NEW_TEMPLATE).take().unwrap();
                Self::new_template(data, template_data);
                ctx.request_update();
                return;
            }
            _ => (),
        }
        child.event(ctx, event, data, env)
    }

    fn lifecycle(
        &mut self,
        child: &mut W,
        ctx: &mut LifeCycleCtx,
        event: &LifeCycle,
        data: &AppData,
        env: &Env,
    ) {
        if let LifeCycle::WidgetAdded = event {
            if let Some(last) = data.recent_templates.iter().next() {
                ctx.submit_command(
                    MSG_THREAD.with(SingleUse::new(Msg::NewTemplateByPath(last.clone()))),
                )
            }
        }
        child.lifecycle(ctx, event, data, env)
    }
}