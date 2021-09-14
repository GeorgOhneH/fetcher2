use std::convert::TryFrom;
use std::fmt::{Debug, Display, Formatter};
use std::sync::{Arc, Mutex, RwLock};

use druid::kurbo::{BezPath, Size};
use druid::piet::{LineCap, LineJoin, RenderContext, StrokeStyle};
use druid::widget::{Controller, Label};
use druid::{
    commands, theme, Command, ExtEventSink, Menu, MenuItem, Rect, Selector, SingleUse, Target,
    WidgetExt, WidgetId, WindowConfig, WindowHandle, WindowLevel,
};
use druid::{
    BoxConstraints, Data, Env, Event, EventCtx, LayoutCtx, Lens, LifeCycle, LifeCycleCtx, PaintCtx,
    Point, UpdateCtx, Widget, WidgetPod,
};

use crate::background_thread::{background_main, NEW_EDIT_TEMPLATE, NEW_TEMPLATE};
use crate::cstruct_window::c_option_window;
use crate::edit_window::{edit_window, EditWindowState};
use crate::settings::{DownloadSettings, Settings};
use crate::template::communication::NODE_EVENT;
use crate::template::nodes::node::NodeEvent;
use crate::template::nodes::node_data::NodeData;
use crate::template::nodes::root_data::RootNodeData;
use crate::template::widget_data::TemplateData;
use crate::template::widget_edit_data::TemplateEditData;
use crate::template::Template;
use crate::ui::TemplateInfoSelect;
use crate::widgets::tree::{DataNodeIndex, NodeIndex, Tree};
use crate::{AppData, Result};
use config::Config;
use directories::{BaseDirs, ProjectDirs, UserDirs};
use druid::commands::{CLOSE_WINDOW, QUIT_APP};
use druid::im::Vector;
use druid_widget_nursery::{selectors, Wedge};
use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};
use std::any::Any;
use std::cmp::max;
use std::collections::HashSet;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::thread::JoinHandle;
use std::{fs, thread};
use tokio::io::AsyncWriteExt;
use url::Position;

selectors! {
    MSG_THREAD: SingleUse<Msg>
}

selectors! {
    OPEN_EDIT
}

selectors! {
    INIT_MAIN_WINDOW_STATE: MainWindowState,
    SAVE_MAIN_WINDOW_STATE: Arc<RwLock<MainWindowState>>,
    INIT_EDIT_WINDOW_STATE: EditWindowState,
    SAVE_EDIT_WINDOW_STATE: Arc<RwLock<EditWindowState>>,
    PARENT_UPDATE_EDIT_WINDOW: SingleUse<SubWindowInfo<EditWindowState >>,
}

selectors! {
    //
    NEW_WIN_INFO: SingleUse<SubWindowInfo<()>>,
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

#[derive(Serialize, Deserialize, Default, Debug, Clone)]
#[serde(default)]
pub struct MainWindowState {
    pub win_size: Option<Size>,
    pub win_pos: Option<Point>,
    pub settings_window: SubWindowInfo<()>,
    pub edit_window: SubWindowInfo<EditWindowState>,

    pub recent_templates: Vec<PathBuf>,
}

pub struct ThreadData {
    pub tx: flume::Sender<Msg>,
    pub handle: SingleUse<JoinHandle<()>>,
}

pub struct MainController {
    thread_data: Option<ThreadData>,
    saved: bool,
    win_state: Arc<RwLock<MainWindowState>>,
}

impl MainController {
    pub fn new(win_state: MainWindowState) -> Self {
        Self {
            thread_data: None,
            saved: false,
            win_state: Arc::new(RwLock::new(win_state)),
        }
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
                let msg = cmd.get_unchecked(MSG_THREAD).take().unwrap();
                self.thread_data
                    .as_ref()
                    .expect("Got Msg before WindowConnected")
                    .tx
                    .send(msg)
                    .unwrap();
                return;
            }
            Event::Command(cmd) if cmd.is(commands::OPEN_FILE) => {
                ctx.set_handled();
                let file_info = cmd.get_unchecked(commands::OPEN_FILE);
                self.thread_data
                    .as_ref()
                    .expect("Got Msg before WindowConnected")
                    .tx
                    .send(Msg::NewTemplateByPath(file_info.path.clone()))
                    .unwrap();
                return;
            }
            Event::WindowConnected => {
                let (tx, rx) = flume::unbounded();
                let sink = ctx.get_external_handle();
                let handle = thread::spawn(move || {
                    background_main(sink, rx);
                });
                self.thread_data = Some(ThreadData {
                    tx,
                    handle: SingleUse::new(handle),
                });

                ctx.submit_command(
                    INIT_MAIN_WINDOW_STATE.with(self.win_state.read().unwrap().clone()),
                );
            }
            Event::WindowCloseRequested => {
                if !self.saved {
                    ctx.set_handled();
                    self.saved = true;
                    ctx.submit_command(Command::new(
                        SAVE_MAIN_WINDOW_STATE,
                        self.win_state.clone(),
                        Target::Window(ctx.window_id()),
                    ));
                    ctx.submit_command(CLOSE_WINDOW)
                } else {
                    let mut win_state = self.win_state.read().unwrap().clone();
                    win_state.win_size = Some(ctx.window().get_size());
                    win_state.win_pos = Some(ctx.window().get_position());
                    // TODO not in main thread
                    let serialized = serde_json::to_string(&win_state).unwrap();

                    fs::create_dir_all(WINDOW_STATE_DIR.as_path().parent().unwrap()).unwrap();

                    let mut f = fs::OpenOptions::new()
                        .write(true)
                        .truncate(true)
                        .create(true)
                        .open(WINDOW_STATE_DIR.as_path())
                        .unwrap();
                    f.write_all(&serialized.as_bytes()).unwrap();
                }
            }
            Event::WindowDisconnected => {
                self.thread_data
                    .as_ref()
                    .expect("Got Msg before WindowConnected")
                    .tx
                    .send(Msg::ExitAndSave)
                    .unwrap();
                self.thread_data
                    .as_ref()
                    .expect("Got Msg before WindowConnected")
                    .handle
                    .take()
                    .unwrap()
                    .join()
                    .unwrap();
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
                data.template.update_node(node_event, &idx);
                return;
            }
            Event::Command(cmd) if cmd.is(NEW_TEMPLATE) => {
                ctx.set_handled();
                let template_data = cmd.get_unchecked(NEW_TEMPLATE).take().unwrap();
                if let Some(new_path) = &template_data.save_path {
                    data.recent_templates.retain(|path| path != new_path);
                    data.recent_templates.push_front(new_path.clone());
                }
                data.template = template_data;
                ctx.request_update();
                return;
            }
            Event::Command(cmd) if cmd.is(INIT_MAIN_WINDOW_STATE) => {
                let main_state = cmd.get_unchecked(INIT_MAIN_WINDOW_STATE);
                data.recent_templates = main_state.recent_templates.clone().into();
                if let Some(path) = data.recent_templates.iter().next() {
                    ctx.submit_command(
                        MSG_THREAD.with(SingleUse::new(Msg::NewTemplateByPath(path.clone()))),
                    )
                }
            }
            Event::Command(cmd) if cmd.is(SAVE_MAIN_WINDOW_STATE) => {
                let main_state = cmd.get_unchecked(SAVE_MAIN_WINDOW_STATE);
                main_state.write().unwrap().recent_templates =
                    data.recent_templates.iter().map(|x| x.clone()).collect();
            }
            _ => (),
        }
        child.event(ctx, event, data, env)
    }
}

pub struct SettingController {
    win_info: SubWindowInfo<()>,
}

impl SettingController {
    pub fn new() -> Self {
        Self {
            win_info: SubWindowInfo::new(()),
        }
    }
}

impl<W: Widget<Option<Settings>>> Controller<Option<Settings>, W> for SettingController {
    fn event(
        &mut self,
        child: &mut W,
        ctx: &mut EventCtx,
        event: &Event,
        data: &mut Option<Settings>,
        env: &Env,
    ) {
        match event {
            Event::WindowConnected => {
                //TODO not in main thread
                if let Ok(file_content) = &fs::read(SETTINGS_DIR.as_path()) {
                    let file_str = String::from_utf8_lossy(file_content);
                    if let Ok(settings) = Settings::load_from_str(file_str.as_ref()) {
                        ctx.submit_command(
                            MSG_THREAD
                                .with(SingleUse::new(Msg::NewSettings(settings.downs.clone()))),
                        );
                        *data = Some(settings);
                    }
                }
            }
            Event::Command(cmd) if cmd.is(INIT_MAIN_WINDOW_STATE) => {
                let main_state = cmd.get_unchecked(INIT_MAIN_WINDOW_STATE);
                self.win_info = main_state.settings_window.clone();
            }
            Event::Command(cmd) if cmd.is(SAVE_MAIN_WINDOW_STATE) => {
                let main_state = cmd.get_unchecked(SAVE_MAIN_WINDOW_STATE);
                main_state.write().unwrap().settings_window = self.win_info.clone();
            }
            Event::Command(cmd) if cmd.is(NEW_WIN_INFO) => {
                self.win_info = cmd.get_unchecked(NEW_WIN_INFO).take().unwrap();
            }
            Event::Command(cmd) if cmd.is(commands::SHOW_PREFERENCES) => {
                ctx.set_handled();
                let (size, pos) = self.win_info.get_size_pos(ctx.window());
                let main_win_id = ctx.window_id();
                let c_window = c_option_window(
                    Some("Settings"),
                    Some(Box::new(
                        move |inner_ctx: &mut EventCtx, old_data, data: &mut Settings| {
                            inner_ctx.submit_command(
                                MSG_THREAD
                                    .with(SingleUse::new(Msg::NewSettings(data.downs.clone())))
                                    .to(main_win_id.clone()),
                            );
                            // TODO not in main thread
                            let serialized = serde_yaml::to_string(&data).unwrap();

                            fs::create_dir_all(SETTINGS_DIR.as_path().parent().unwrap()).unwrap();

                            let mut f = fs::OpenOptions::new()
                                .write(true)
                                .truncate(true)
                                .create(true)
                                .open(SETTINGS_DIR.as_path())
                                .unwrap();
                            f.write_all(&serialized.as_bytes()).unwrap();
                        },
                    )),
                )
                .controller(SubStateController::new((), ctx.widget_id()));
                ctx.new_sub_window(
                    WindowConfig::default()
                        .show_titlebar(true)
                        .window_size(size)
                        .set_position(pos)
                        .set_level(WindowLevel::Modal),
                    c_window,
                    data.clone(),
                    env.clone(),
                );

                return;
            }
            _ => (),
        }
        child.event(ctx, event, data, env)
    }
}

pub struct EditController {
    current_data: TemplateEditData,
    win_info: SubWindowInfo<EditWindowState>,
}

impl EditController {
    pub fn new() -> Self {
        Self {
            current_data: TemplateEditData::new(),
            win_info: SubWindowInfo::new(Default::default()),
        }
    }
    fn make_sub_window(&self, ctx: &mut EventCtx, env: &Env, edit_data: TemplateEditData) {
        let (size, pos) = self.win_info.get_size_pos(ctx.window());
        let window = edit_window(edit_data).controller(SubStateController::new(
            self.win_info.data_state.clone(),
            ctx.widget_id(),
        ));
        ctx.new_sub_window(
            WindowConfig::default()
                .show_titlebar(true)
                .window_size(size)
                .set_position(pos)
                .set_level(WindowLevel::Modal),
            window,
            (),
            env.clone(),
        );
    }
}

impl<W: Widget<()>> Controller<(), W> for EditController {
    fn event(
        &mut self,
        child: &mut W,
        ctx: &mut EventCtx,
        event: &Event,
        data: &mut (),
        env: &Env,
    ) {
        match event {
            Event::Command(cmd) if cmd.is(INIT_MAIN_WINDOW_STATE) => {
                let main_state = cmd.get_unchecked(INIT_MAIN_WINDOW_STATE);
                self.win_info = main_state.edit_window.clone();
            }
            Event::Command(cmd) if cmd.is(SAVE_MAIN_WINDOW_STATE) => {
                let main_state = cmd.get_unchecked(SAVE_MAIN_WINDOW_STATE);
                main_state.write().unwrap().edit_window = self.win_info.clone();
            }
            Event::Command(cmd) if cmd.is(NEW_EDIT_TEMPLATE) => {
                ctx.set_handled();
                let edit_data = cmd.get_unchecked(NEW_EDIT_TEMPLATE).take().unwrap();
                self.current_data = edit_data;
                return;
            }
            Event::Command(cmd) if cmd.is(OPEN_EDIT) => {
                ctx.set_handled();
                self.make_sub_window(ctx, env, self.current_data.clone());
                return;
            }
            Event::Command(cmd) if cmd.is(commands::NEW_FILE) => {
                ctx.set_handled();
                let edit_data = TemplateEditData::new();
                self.make_sub_window(ctx, env, edit_data);
                return;
            }
            Event::Command(cmd) if cmd.is(PARENT_UPDATE_EDIT_WINDOW) => {
                ctx.set_handled();
                let new_win_state = cmd.get_unchecked(PARENT_UPDATE_EDIT_WINDOW).take().unwrap();
                self.win_info = new_win_state;
                return;
            }
            _ => (),
        }
        child.event(ctx, event, data, env)
    }
}

#[derive(Serialize, Deserialize, Default, Debug, Clone)]
#[serde(default)]
pub struct SubWindowInfo<T: Clone + Default + Debug + Serialize> {
    data_state: T,
    win_state: Option<WindowState>,
}

impl<T: Clone + Default + Debug + Serialize> SubWindowInfo<T> {
    pub fn new(data_state: T) -> Self {
        Self {
            data_state,
            win_state: None,
        }
    }
    pub fn with_win_state(data_state: T, size: Size, pos: Point) -> Self {
        Self {
            data_state,
            win_state: Some(WindowState::new(size, pos)),
        }
    }

    pub fn get_size_pos(&self, win_handle: &WindowHandle) -> (Size, Point) {
        if let Some(win_state) = &self.win_state {
            return (win_state.size, win_state.pos);
        }
        let win_pos = win_handle.get_position();
        let (win_size_w, win_size_h) = win_handle.get_size().into();
        let (size_w, size_h) = (f64::min(600., win_size_w), f64::min(600., win_size_h));
        let pos = win_pos + ((win_size_w - size_w) / 2., (win_size_h - size_h) / 2.);
        (Size::new(size_w, size_h), pos)
    }
}

#[derive(Serialize, Deserialize, Default, Debug, Clone)]
#[serde(default)]
pub struct WindowState {
    size: Size,
    pos: Point,
}

impl WindowState {
    pub fn new(size: Size, pos: Point) -> Self {
        Self { size, pos }
    }
}

pub struct SubStateController<T> {
    win_state: Arc<RwLock<T>>,
    saved: bool,
    parent_id: WidgetId,
}

impl<T> SubStateController<T> {
    pub fn new(win_state: T, parent_id: WidgetId) -> Self {
        Self {
            win_state: Arc::new(RwLock::new(win_state)),
            saved: false,
            parent_id,
        }
    }
}

impl<T: Data, W: Widget<T>> Controller<T, W> for SubStateController<EditWindowState> {
    fn event(&mut self, child: &mut W, ctx: &mut EventCtx, event: &Event, data: &mut T, env: &Env) {
        match event {
            Event::WindowConnected => ctx.submit_command(
                INIT_EDIT_WINDOW_STATE.with(self.win_state.read().unwrap().clone()),
            ),
            Event::WindowCloseRequested => {
                if !self.saved {
                    ctx.set_handled();
                    self.saved = true;
                    ctx.submit_command(Command::new(
                        SAVE_EDIT_WINDOW_STATE,
                        self.win_state.clone(),
                        Target::Window(ctx.window_id()),
                    ));
                    ctx.submit_command(CLOSE_WINDOW)
                } else {
                    let win_info = SubWindowInfo::with_win_state(
                        self.win_state.read().unwrap().clone(),
                        ctx.window().get_size(),
                        ctx.window().get_position(),
                    );
                    ctx.submit_command(
                        PARENT_UPDATE_EDIT_WINDOW
                            .with(SingleUse::new(win_info))
                            .to(self.parent_id),
                    )
                }
            }
            _ => (),
        }
        child.event(ctx, event, data, env)
    }
}

impl<T: Data, W: Widget<T>> Controller<T, W> for SubStateController<()> {
    fn event(&mut self, child: &mut W, ctx: &mut EventCtx, event: &Event, data: &mut T, env: &Env) {
        if let Event::WindowCloseRequested = event {
            let win_info = SubWindowInfo::with_win_state(
                self.win_state.read().unwrap().clone(),
                ctx.window().get_size(),
                ctx.window().get_position(),
            );
            ctx.submit_command(
                NEW_WIN_INFO
                    .with(SingleUse::new(win_info))
                    .to(self.parent_id),
            )
        }
        child.event(ctx, event, data, env)
    }
}
