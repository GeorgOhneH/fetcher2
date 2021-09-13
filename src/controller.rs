use std::convert::TryFrom;
use std::fmt::{Debug, Display, Formatter};
use std::sync::Arc;

use druid::kurbo::{BezPath, Size};
use druid::piet::{LineCap, LineJoin, RenderContext, StrokeStyle};
use druid::widget::{Controller, Label};
use druid::{
    commands, theme, ExtEventSink, Menu, MenuItem, Rect, Selector, SingleUse, WidgetExt, WidgetId,
    WindowConfig, WindowLevel,
};
use druid::{
    BoxConstraints, Data, Env, Event, EventCtx, LayoutCtx, Lens, LifeCycle, LifeCycleCtx, PaintCtx,
    Point, UpdateCtx, Widget, WidgetPod,
};

use crate::background_thread::{NEW_TEMPLATE, EDIT_DATA};
use crate::cstruct_window::c_option_window;
use crate::delegate::{Msg, MSG_THREAD};
use crate::settings::Settings;
use crate::template::communication::NODE_EVENT;
use crate::template::nodes::node::NodeEvent;
use crate::template::nodes::node_data::NodeData;
use crate::template::nodes::root_data::RootNodeData;
use crate::template::widget_data::TemplateData;
use crate::template::Template;
use crate::ui::TemplateInfoSelect;
use crate::widgets::tree::{DataNodeIndex, NodeIndex, Tree};
use crate::{AppData, Result};
use druid::im::Vector;
use druid_widget_nursery::{selectors, Wedge};
use std::cmp::max;
use std::collections::HashSet;
use std::path::{Path, PathBuf};
use crate::edit_window::edit_window;

pub struct TemplateController;

impl<W: Widget<TemplateData>> Controller<TemplateData, W> for TemplateController {
    fn event(
        &mut self,
        child: &mut W,
        ctx: &mut EventCtx,
        event: &Event,
        data: &mut TemplateData,
        env: &Env,
    ) {
        match event {
            Event::Command(cmd) if cmd.is(NODE_EVENT) => {
                ctx.set_handled();
                let (node_event, idx) = cmd.get_unchecked(NODE_EVENT).take().unwrap();
                data.update_node(node_event, &idx);
                return;
            }
            Event::Command(cmd) if cmd.is(NEW_TEMPLATE) => {
                ctx.set_handled();
                let template_data = cmd.get_unchecked(NEW_TEMPLATE).take().unwrap();
                *data = template_data;
                ctx.request_update();
                return;
            }
            _ => (),
        }
        child.event(ctx, event, data, env)
    }
}

pub struct SettingController;

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
            Event::Command(cmd) if cmd.is(commands::SHOW_PREFERENCES) => {
                ctx.set_handled();
                let window = ctx.window();
                let win_pos = window.get_position();
                let (win_size_w, win_size_h) = window.get_size().into();
                let (size_w, size_h) = (f64::min(600., win_size_w), f64::min(600., win_size_h));
                let pos = win_pos + ((win_size_w - size_w) / 2., (win_size_h - size_h) / 2.);
                let main_win_id = ctx.window_id();
                let c_window = c_option_window(Some(Box::new(
                    move |inner_ctx: &mut EventCtx, old_data, data: &mut Settings| {
                        inner_ctx.submit_command(
                            MSG_THREAD
                                .with(SingleUse::new(Msg::NewSettings(data.downs.clone())))
                                .to(main_win_id.clone()),
                        )
                    },
                )));
                ctx.new_sub_window(
                    WindowConfig::default()
                        .show_titlebar(true)
                        .window_size(Size::new(size_w, size_h))
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


pub struct EditController;

impl<W: Widget<()>> Controller<(), W> for EditController {
    fn event(&mut self, child: &mut W, ctx: &mut EventCtx, event: &Event, data: &mut (), env: &Env) {
        match event {
            Event::Command(cmd) if cmd.is(EDIT_DATA) => {
                ctx.set_handled();
                let edit_data = cmd.get_unchecked(EDIT_DATA).take().unwrap();
                let window = ctx.window();
                let win_pos = window.get_position();
                let (win_size_w, win_size_h) = window.get_size().into();
                let (size_w, size_h) = (f64::min(600., win_size_w), f64::min(600., win_size_h));
                let pos = win_pos + ((win_size_w - size_w) / 2., (win_size_h - size_h) / 2.);
                ctx.new_sub_window(
                    WindowConfig::default()
                        .show_titlebar(true)
                        .window_size(Size::new(size_w, size_h))
                        .set_position(pos)
                        .set_level(WindowLevel::Modal),
                    edit_window(edit_data),
                    (),
                    env.clone(),
                );

                return;
            }
            _ => (),
        }
        child.event(ctx, event, data, env)
    }
}