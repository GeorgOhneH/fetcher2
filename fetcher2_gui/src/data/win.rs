use druid::kurbo::Size;
use druid::{Data, Point};
use druid::{Scalable, WindowHandle};
use serde::{Deserialize, Serialize};
use std::fmt::Debug;

#[derive(Serialize, Deserialize, Debug, Clone, Data, Default)]
pub struct SubWindowInfo<T> {
    pub data: T,

    pub win_state: Option<WindowState>,
}

impl<T> SubWindowInfo<T> {
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

#[derive(Serialize, Deserialize, Debug, Clone, Data)]
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
