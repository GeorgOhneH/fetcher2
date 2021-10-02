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
    commands, theme, Command, ExtEventSink, Menu, MenuItem,
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
