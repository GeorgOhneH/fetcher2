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

use crate::controller::{
    EditController, MainController, Msg, SettingController, TemplateController, MSG_THREAD,
    OPEN_EDIT,
};
use crate::cstruct_window::{c_option_window, CStructBuffer};

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
    Maybe, Padding, Scroll, SizedBox, Spinner, Split, Switch, TextBox, ViewSwitcher,
};
use druid::{
    commands, im, menu, AppDelegate, AppLauncher, Application, Color, Command, Data, DelegateCtx,
    Env, Event, EventCtx, ExtEventSink, FileInfo, Handled, LayoutCtx, Lens, LifeCycle,
    LifeCycleCtx, LocalizedString, Menu, MenuItem, MouseButton, PaintCtx, Point, Screen, Selector,
    SingleUse, Size, SysMods, Target, UnitPoint, UpdateCtx, Vec2, Widget, WidgetExt, WidgetId,
    WidgetPod, WindowConfig, WindowDesc, WindowId, WindowLevel,
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
    #[data(eq)]
    pub recent_templates: Vector<PathBuf>,
    pub settings: Option<Settings>,
    pub template_info_select: TemplateInfoSelect,
}

pub fn make_menu(_: Option<WindowId>, data: &AppData, _: &Env) -> Menu<AppData> {
    let mut base = Menu::empty();

    let mut open_recent = Menu::new("Open Recent");
    for path in data.recent_templates.iter() {
        if let Some(file_name) = path.file_name() {
            let path_clone = path.clone();
            open_recent = open_recent.entry(
                MenuItem::new(file_name.to_string_lossy().to_string()).on_activate(
                    move |ctx, data: &mut AppData, env| {
                        ctx.submit_command(commands::OPEN_FILE.with(FileInfo {
                            path: path_clone.clone(),
                            format: None,
                        }))
                    },
                ),
            )
        }
    }

    #[cfg(target_os = "macos")]
    {
        base = Menu::new(LocalizedString::new(""))
            .entry(
                Menu::new(LocalizedString::new("macos-menu-application-menu"))
                    .entry(menu::sys::mac::application::preferences())
                    .separator()
                    .entry(menu::sys::mac::application::hide())
                    .entry(menu::sys::mac::application::hide_others()),
            )
            .entry(
                Menu::new(LocalizedString::new("common-menu-file-menu"))
                    .entry(menu::sys::mac::file::new_file())
                    .entry(menu::sys::mac::file::open_file())
                    .entry(open_recent)
                    .separator()
                    .entry(
                        MenuItem::new("Open Edit")
                            .command(OPEN_EDIT)
                            .hotkey(SysMods::Cmd, "e"),
                    ),
            );
    }
    #[cfg(any(target_os = "windows", target_os = "linux"))]
    {
        base = base.entry(
            Menu::new("Hello")
                .entry(menu::sys::win::file::new())
                .entry(menu::sys::win::file::open())
                .entry(open_recent)
                .separator()
                .entry(
                    MenuItem::new("Open Edit")
                        .command(OPEN_EDIT)
                        .hotkey(SysMods::Cmd, "e"),
                )
                .separator()
                .entry(
                    MenuItem::new("Settings")
                        .command(commands::SHOW_PREFERENCES)
                        .hotkey(SysMods::Cmd, "d"),
                ),
        );
    }

    base.rebuild_on(|old_data, data, env| old_data.recent_templates != data.recent_templates)
}
pub fn build_ui() -> impl Widget<AppData> {
    Flex::column()
        .cross_axis_alignment(CrossAxisAlignment::Start)
        .with_child(
            SizedBox::empty()
                .controller(TemplateController::new())
                .padding(0.)
        )
        .with_child(
            SizedBox::empty()
                .controller(SettingController::new())
                .padding(0.)
                .lens(AppData::settings),
        )
        .with_child(
            SizedBox::empty()
                .controller(EditController::new())
                .padding(0.)
                .lens(lens::Unit),
        )
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
                    .lens(AppData::template),
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
            Target::Window(ctx.window_id()),
        ));
        // ctx.submit_command(Command::new(MSG_THREAD, SingleUse::new(Msg::Cancel), Target::Global))
    });
    let stop = Button::new("Stop").on_click(|ctx, _, _| {
        ctx.submit_command(Command::new(
            MSG_THREAD,
            SingleUse::new(Msg::Cancel),
            Target::Window(ctx.window_id()),
        ))
    });
    let edit = Button::new("Edit").on_click(|ctx, _, _| ctx.submit_command(OPEN_EDIT));
    let settings = Button::new("Settings")
        .on_click(|ctx, _, env| ctx.submit_command(commands::SHOW_PREFERENCES));

    Flex::row()
        .cross_axis_alignment(CrossAxisAlignment::Start)
        .with_child(start)
        .with_default_spacer()
        .with_child(stop)
        .with_default_spacer()
        .with_child(edit)
        .with_default_spacer()
        .with_child(settings)
}
