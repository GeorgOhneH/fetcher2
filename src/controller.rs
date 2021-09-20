use std::any::Any;
use std::cmp::max;
use std::collections::HashSet;
use std::convert::TryFrom;
use std::fmt::{Debug, Display, Formatter};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex, RwLock};
use std::thread::JoinHandle;
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
use crate::edit_window::{edit_window, EditWindowData};
use crate::settings::{DownloadSettings, Settings};
use crate::template::communication::NODE_EVENT;
use crate::template::nodes::node::NodeEvent;
use crate::template::nodes::node_data::NodeData;
use crate::template::nodes::root_data::RootNodeData;
use crate::template::widget_data::TemplateData;
use crate::template::widget_edit_data::TemplateEditData;
use crate::template::Template;
use crate::ui::{OptionSettings, TemplateInfoSelect};
use crate::utils::show_err;
use crate::widgets::sub_window_widget::SubWindow;
use crate::widgets::tree::{DataNodeIndex, NodeIndex, Tree};
use crate::{AppData, Result, TError};
use std::time::Duration;

selectors! {
    MSG_THREAD: SingleUse<Msg>
}

selectors! {
    OPEN_EDIT
}

lazy_static! {
    pub static ref CONFIG_DIR: PathBuf = ProjectDirs::from("ch", "fetcher2", "fetcher2")
        .expect("Could not find a place to store the config files")
        .config_dir()
        .to_owned();
    pub static ref SETTINGS_DIR: PathBuf = Path::join(CONFIG_DIR.as_path(), "settings.json");
    pub static ref WINDOW_STATE_DIR: PathBuf =
        Path::join(CONFIG_DIR.as_path(), "window_state.json");
}

#[derive(Debug)]
pub enum Msg {
    StartAll,
    StartByIndex(HashSet<NodeIndex>),
    Cancel,
    NewSettings(DownloadSettings),
    NewTemplate(Template),
    NewTemplateByPath(PathBuf),
    ExitAndSave,
}

pub struct MainController {
    tx: flume::Sender<Msg>,
    load_err: Option<TError>,
}

impl MainController {
    pub fn new(load_err: Option<TError>, tx: flume::Sender<Msg>) -> Self {
        Self { tx, load_err }
    }
}

impl MainController {
    fn handle_thread_msg(ctx: &mut EventCtx, data: &AppData, env: &Env, thread_msg: ThreadMsg) {
        match thread_msg {
            ThreadMsg::SettingsRequired => ctx.submit_command(commands::SHOW_PREFERENCES),
            ThreadMsg::TemplateLoadingError(err) => {
                show_err(ctx, data, env, err, "Could not load template")
            }
            ThreadMsg::TemplateSaveError(err) => {
                show_err(ctx, data, env, err, "Could not save template")
            }
        };
    }

    fn save_window_state(app_state: &AppData) -> Result<()> {
        let serialized = ron::to_string(app_state)?;

        fs::create_dir_all(WINDOW_STATE_DIR.as_path().parent().expect(""))?;

        let mut f = fs::OpenOptions::new()
            .write(true)
            .truncate(true)
            .create(true)
            .open(WINDOW_STATE_DIR.as_path())?;
        f.write_all(&serialized.as_bytes())?;
        Ok(())
    }
}

impl<W: Widget<AppData>> Controller<AppData, W> for MainController {
    fn event(
        &mut self,
        child: &mut W,
        ctx: &mut EventCtx,
        event: &Event,
        data: &mut AppData,
        env: &Env,
    ) {
        match event {
            Event::Command(cmd) if cmd.is(MSG_THREAD) => {
                ctx.set_handled();
                let msg = cmd.get_unchecked(MSG_THREAD).take().expect("");
                self.tx.send(msg).unwrap();
                return;
            }
            Event::Command(cmd) if cmd.is(MSG_FROM_THREAD) => {
                ctx.set_handled();
                let thread_msg = cmd.get_unchecked(MSG_FROM_THREAD).take().expect("");
                Self::handle_thread_msg(ctx, data, env, thread_msg)
            }
            Event::Command(cmd) if cmd.is(commands::OPEN_FILE) => {
                ctx.set_handled();
                let file_info = cmd.get_unchecked(commands::OPEN_FILE);
                self.tx
                    .send(Msg::NewTemplateByPath(file_info.path.clone()))
                    .expect("");
                return;
            }
            Event::WindowConnected => {
                ctx.request_timer(Duration::from_millis(100));
            }
            Event::Timer(_) => {
                if let Some(err) = self.load_err.take() {
                    show_err(ctx, data, env, err, "Could not load window state");
                }
            }
            Event::WindowCloseRequested => {
                data.main_window = Some(WindowState::from_win(ctx.window()));
            }
            Event::WindowDisconnected => {
                self.tx.send(Msg::ExitAndSave).expect("");
                Self::save_window_state(data).expect("Could not save AppData")
            }
            _ => (),
        }
        child.event(ctx, event, data, env)
    }
}

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

pub struct EditController {
}

impl EditController {
    pub fn new() -> Self {
        Self {
        }
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
                let edit_data = cmd.get_unchecked(NEW_EDIT_TEMPLATE).take().unwrap();
                data.data.edit_template = edit_data;
                return;
            }
            Event::Command(cmd) if cmd.is(OPEN_EDIT) => {
                ctx.set_handled();
                self.make_sub_window(ctx, env, &data, false);
                return;
            }
            Event::Command(cmd) if cmd.is(commands::NEW_FILE) => {
                ctx.set_handled();
                self.make_sub_window(ctx, env, &data, true);
                return;
            }
            _ => (),
        }
        child.event(ctx, event, data, env)
    }
}

#[derive(Config, Debug, Clone, Data)]
pub struct SubWindowInfo<T> {
    #[config(ty = "struct")]
    pub data: T,

    #[config(ty = "_<struct>")]
    pub win_state: Option<WindowState>,
}

impl<T: Clone + Debug + Config> SubWindowInfo<T> {
    pub fn new(data_state: T) -> Self {
        Self {
            data: data_state,
            win_state: None,
        }
    }
    pub fn with_win_state(data_state: T, size: Size, pos: Point) -> Self {
        Self {
            data: data_state,
            win_state: Some(WindowState::new(size, pos)),
        }
    }

    pub fn get_size_pos(&self, win_handle: &WindowHandle) -> (Size, Point) {
        if let Some(win_state) = &self.win_state {
            return (win_state.get_size(), win_state.get_pos());
        }
        WindowState::default_size_pos(win_handle)
    }
}

#[derive(Config, Debug, Clone, Data)]
pub struct WindowState {
    // TODO
    size_w: isize,
    size_h: isize,

    pos_x: isize,
    pos_y: isize,
}

impl WindowState {
    pub fn new(size: Size, pos: Point) -> Self {
        Self {
            size_w: size.width as isize,
            size_h: size.height as isize,
            pos_x: pos.x as isize,
            pos_y: pos.y as isize,
        }
    }

    pub fn get_size(&self) -> Size {
        Size::new(self.size_w as f64, self.size_h as f64)
    }

    pub fn get_pos(&self) -> Point {
        Point::new(self.pos_x as f64, self.pos_y as f64)
    }

    pub fn from_win(handle: &WindowHandle) -> Self {
        // TODO not panic
        let scale = handle.get_scale().unwrap();
        Self::new(handle.get_size().to_dp(scale), handle.get_position())
    }
    pub fn default_size_pos(win_handle: &WindowHandle) -> (Size, Point) {
        let (win_size_w, win_size_h) = win_handle.get_size().into();
        let (size_w, size_h) = (f64::min(600., win_size_w), f64::min(600., win_size_h));
        let pos = ((win_size_w - size_w) / 2., (win_size_h - size_h) / 2.);
        (Size::new(size_w, size_h), pos.into())
    }
}
