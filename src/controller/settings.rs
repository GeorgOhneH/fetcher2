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
use crate::data::settings::{OptionSettings, Settings};
use crate::data::win::SubWindowInfo;
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

pub struct SettingController {}

impl SettingController {
    pub fn new() -> Self {
        Self {}
    }

    fn show_settings(&self, ctx: &mut EventCtx, data: &SubWindowInfo<OptionSettings>, env: &Env) {
        let (size, pos) = data.get_size_pos(ctx.window());
        let main_win_id = ctx.window_id();
        let c_window = c_option_window(
            Some("Settings"),
            Some(Box::new(
                move |inner_ctx: &mut EventCtx, old_data, data: &mut Settings, env| {
                    inner_ctx.submit_command(
                        MSG_THREAD
                            .with(SingleUse::new(Msg::NewSettings(data.download.clone())))
                            .to(main_win_id.clone()),
                    );
                },
            )),
        )
            .lens(OptionSettings::settings);
        ctx.new_sub_window(
            WindowConfig::default()
                .show_titlebar(true)
                .window_size(size)
                .set_position(pos)
                .set_level(WindowLevel::Modal),
            SubWindow::new(c_window),
            data.clone(),
            env.clone(),
        );
    }
}

impl<W: Widget<SubWindowInfo<OptionSettings>>> Controller<SubWindowInfo<OptionSettings>, W>
for SettingController
{
    fn event(
        &mut self,
        child: &mut W,
        ctx: &mut EventCtx,
        event: &Event,
        data: &mut SubWindowInfo<OptionSettings>,
        env: &Env,
    ) {
        match event {
            Event::Command(cmd) if cmd.is(commands::SHOW_PREFERENCES) => {
                ctx.set_handled();
                self.show_settings(ctx, data, env);
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
        data: &SubWindowInfo<OptionSettings>,
        env: &Env,
    ) {
        if let LifeCycle::WidgetAdded = event {
            if let Some(settings) = &data.data.settings {
                ctx.submit_command(
                    MSG_THREAD.with(SingleUse::new(Msg::NewSettings(settings.download.clone()))),
                );
            }
        }
        child.lifecycle(ctx, event, data, env)
    }
}
