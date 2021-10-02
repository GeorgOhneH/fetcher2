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
use std::{fs, thread};

use config::{Config, InvalidError, RequiredError};
use directories::{BaseDirs, ProjectDirs, UserDirs};
use druid::commands::{CLOSE_WINDOW, QUIT_APP};
use druid::im::Vector;
use druid::kurbo::{BezPath, Size};
use druid::piet::{LineCap, LineJoin, RenderContext, StrokeStyle};
use druid::widget::{Controller, Label};
use druid::{
    commands, theme, Command, ExtEventSink, HasRawWindowHandle, Menu, MenuItem, RawWindowHandle,
    Rect, Scalable, Selector, SingleUse, Target, WidgetExt, WidgetId, WindowConfig, WindowHandle,
    WindowLevel,
};
use druid::{
    BoxConstraints, Data, Env, Event, EventCtx, LayoutCtx, Lens, LifeCycle, LifeCycleCtx, PaintCtx,
    Point, UpdateCtx, Widget, WidgetPod,
};
use druid_widget_nursery::{selectors, Wedge};
use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};
use tokio::io::AsyncWriteExt;
use url::Position;

use crate::background_thread::{
    background_main, ThreadMsg, MSG_FROM_THREAD, NEW_EDIT_TEMPLATE, NEW_TEMPLATE,
};
use crate::cstruct_window::c_option_window;
use crate::data::edit::EditWindowData;
use crate::data::win::SubWindowInfo;
use crate::edit_window::edit_window;
use crate::template::communication::NODE_EVENT;
use crate::template::nodes::node::NodeEvent;
use crate::template::nodes::node_data::NodeData;
use crate::template::nodes::root_data::RootNodeData;
use crate::template::widget_data::TemplateData;
use crate::template::widget_edit_data::TemplateEditData;
use crate::template::Template;
use crate::utils::show_err;
use crate::widgets::sub_window_widget::SubWindow;
use crate::widgets::tree::{DataNodeIndex, NodeIndex, Tree};
use crate::{Result, TError};

selectors! {
    OPEN_EDIT
}

pub struct EditController {}

impl EditController {
    pub fn new() -> Self {
        Self {}
    }
    fn make_sub_window(
        &self,
        ctx: &mut EventCtx,
        env: &Env,
        data: &SubWindowInfo<EditWindowData>,
        new: bool,
    ) {
        let (size, pos) = data.get_size_pos(ctx.window());
        let window = edit_window(new);
        ctx.new_sub_window(
            WindowConfig::default()
                .show_titlebar(true)
                .window_size(size)
                .set_position(pos)
                .set_level(WindowLevel::Modal),
            SubWindow::new(window),
            data.clone(),
            env.clone(),
        );
    }
}

impl<W: Widget<SubWindowInfo<EditWindowData>>> Controller<SubWindowInfo<EditWindowData>, W>
    for EditController
{
    fn event(
        &mut self,
        child: &mut W,
        ctx: &mut EventCtx,
        event: &Event,
        data: &mut SubWindowInfo<EditWindowData>,
        env: &Env,
    ) {
        match event {
            Event::Command(cmd) if cmd.is(NEW_EDIT_TEMPLATE) => {
                ctx.set_handled();
                let (edit_root, path) = cmd.get_unchecked(NEW_EDIT_TEMPLATE).take().unwrap();
                data.data.edit_template.root = edit_root;
                data.data.edit_template.save_path = path;
                return;
            }
            Event::Command(cmd) if cmd.is(OPEN_EDIT) => {
                ctx.set_handled();
                self.make_sub_window(ctx, env, data, false);
                return;
            }
            Event::Command(cmd) if cmd.is(commands::NEW_FILE) => {
                ctx.set_handled();
                self.make_sub_window(ctx, env, data, true);
                return;
            }
            _ => (),
        }
        child.event(ctx, event, data, env)
    }
}
