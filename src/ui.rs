use crate::settings::{DownloadSettings, Settings, Test};
use crate::template::{DownloadArgs, Extensions, Mode, Template};
use config::{CBool, CInteger, CKwarg, CPath, CString, CType, Config};
use config_derive::Config;
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

use crate::background_thread::EDIT_DATA;
use crate::cstruct_window::{CStructBuffer, c_option_window};
use crate::delegate::{Msg, TemplateDelegate, MSG_THREAD};
use crate::edit_window::edit_window;
use crate::template::communication::RawCommunication;
use crate::template::nodes::node_data::NodeData;
use crate::template::nodes::root_data::RootNodeData;
use crate::template::widget_data::TemplateData;
use crate::widgets::file_watcher::FileWatcher;
use crate::widgets::header::Header;
use crate::widgets::tree::Tree;
use crate::widgets::widget_ext::WidgetExt as _;
use config::CStruct;
use config::State;
use druid::im::{vector, Vector};
use druid::lens::{self, InArc, LensExt};
use druid::text::{Formatter, ParseFormatter, Selection, Validation, ValidationError};
use druid::widget::{
    Button, Checkbox, Controller, CrossAxisAlignment, Either, Flex, Label, LineBreaking, List,
    Maybe, Scroll, SizedBox, Spinner, Split, Switch, TextBox, ViewSwitcher,
};
use druid::{
    im, AppDelegate, AppLauncher, Application, Color, Command, Data, DelegateCtx, Env, Event,
    EventCtx, ExtEventSink, Handled, LayoutCtx, Lens, LifeCycle, LifeCycleCtx, LocalizedString,
    MouseButton, PaintCtx, Point, Screen, Selector, SingleUse, Size, Target, UnitPoint, UpdateCtx,
    Vec2, Widget, WidgetExt, WidgetId, WidgetPod, WindowConfig, WindowDesc, WindowLevel,
};
use druid_widget_nursery::WidgetExt as _;
use flume;
use futures::future::BoxFuture;
use std::any::Any;
use std::cmp::max;
use std::collections::HashMap;
use std::future::Future;
use std::option::Option::Some;
use std::path::PathBuf;
use std::pin::Pin;
use tokio::time;
use tokio::time::Duration;

#[derive(Clone, Copy, Debug, Data, PartialEq)]
pub enum TemplateInfoSelect {
    Nothing,
    General,
    Folder,
    History,
}

#[derive(Clone, Lens, Debug, Data)]
pub struct AppData {
    pub template: TemplateData,
    pub settings: Option<Settings>,
    pub template_info_select: TemplateInfoSelect,
    pub test: Option<Test>,
}

pub fn build_ui() -> impl Widget<AppData> {
    Flex::column()
        .cross_axis_alignment(CrossAxisAlignment::Start)
        .with_child(tool_bar())
        .with_flex_child(template_ui(), 1.)
        .padding(10.)
    // .debug_paint_layout()
}

fn template_ui() -> impl Widget<AppData> {
    Flex::column()
        .with_flex_child(
            Split::rows(
                TemplateData::build_widget()
                    .border(Color::WHITE, 1.)
                    .lens(AppData::template)
                ,
                info_view_ui(),
            )
            .draggable(true)
            .expand_width(),
            1.,
        )
        .with_child(info_view_selector_ui().lens(AppData::template_info_select))
}

fn info_view_ui() -> impl Widget<AppData> {
    ViewSwitcher::new(
        |data: &AppData, _env| data.template_info_select,
        |selector, _data, _env| match selector {
            TemplateInfoSelect::General => Label::new("General").boxed(),
            TemplateInfoSelect::Folder => FileWatcher::new(|data: &AppData| match &data.settings {
                Some(settings) if data.template.root.selected.len() > 0 => {
                    let idx = &data.template.root.selected[0];
                    let node = data
                        .template
                        .node(&idx.clone().into_iter().collect::<Vec<_>>());
                    node.path
                        .as_ref()
                        .map(|path| settings.downs.save_path.join(path))
                }
                _ => None,
            })
            .boxed(),
            TemplateInfoSelect::History => Label::new("History").boxed(),
            TemplateInfoSelect::Nothing => SizedBox::empty().boxed(),
        },
    )
}

fn info_view_selector_ui() -> impl Widget<TemplateInfoSelect> {
    Flex::row()
        .with_child(
            Button::new("General").on_click(|ctx, data: &mut TemplateInfoSelect, _env| {
                *data = TemplateInfoSelect::General
            }),
        )
        .with_child(
            Button::new("Folder").on_click(|ctx, data: &mut TemplateInfoSelect, _env| {
                *data = TemplateInfoSelect::Folder
            }),
        )
        .with_child(
            Button::new("History").on_click(|ctx, data: &mut TemplateInfoSelect, _env| {
                *data = TemplateInfoSelect::History
            }),
        )
}

fn tool_bar() -> impl Widget<AppData> {
    let start = Button::new("Start").on_click(|ctx, _, _| {
        ctx.submit_command(Command::new(
            MSG_THREAD,
            SingleUse::new(Msg::StartAll),
            Target::Global,
        ));
        // ctx.submit_command(Command::new(MSG_THREAD, SingleUse::new(Msg::Cancel), Target::Global))
    });
    let stop = Button::new("Stop").on_click(|ctx, _, _| {
        ctx.submit_command(Command::new(
            MSG_THREAD,
            SingleUse::new(Msg::Cancel),
            Target::Global,
        ))
    });
    let edit = Button::new("Edit")
        .on_click(|ctx, _: &mut (), _| {
            ctx.submit_command(Command::new(
                MSG_THREAD,
                SingleUse::new(Msg::RequestEditData(ctx.widget_id())),
                Target::Global,
            ))
        })
        .on_command2(EDIT_DATA, |ctx, command_data, data: &mut (), env| {
            ctx.set_handled();

            let edit_data = command_data.take().unwrap();
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
        })
        .padding(0.)
        .lens(lens::Unit);
    let settings = Button::new("Settings")
        .on_click(|ctx, data: &mut Option<Settings>, env| {
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
                c_option_window(),
                data.clone(),
                env.clone(),
            );
        })
        .padding(0.) // So it's enclosed in a WidgetPod, (just a nop)
        .on_change(
            |ctx, old_data: &Option<Settings>, data: &mut Option<Settings>, _env| {
                if let Some(settings) = data {
                    ctx.submit_command(
                        MSG_THREAD.with(SingleUse::new(Msg::NewSettings(settings.downs.clone()))),
                    )
                }
            },
        )
        .lens(AppData::settings);
    let test = Button::new("Test")
        .on_click(|ctx, data: &mut Option<Test>, env| {
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
                c_option_window(),
                data.clone(),
                env.clone(),
            );
        })
        .padding(0.) // So it's enclosed in a WidgetPod, (just a nop)
        .lens(AppData::test);

    Flex::row()
        .cross_axis_alignment(CrossAxisAlignment::Start)
        .with_child(start)
        .with_default_spacer()
        .with_child(stop)
        .with_default_spacer()
        .with_child(edit)
        .with_default_spacer()
        .with_child(settings)
        .with_default_spacer()
        .with_child(test)
}
