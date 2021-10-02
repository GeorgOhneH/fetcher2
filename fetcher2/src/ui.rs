use std::any::Any;
use std::cmp::max;
use std::collections::HashMap;
use std::collections::HashSet;
use std::error::Error;
use std::future::Future;
use std::ops::{Deref, DerefMut};
use std::option::Option::Some;
use std::path::Path;
use std::path::PathBuf;
use std::pin::Pin;
use std::sync::{Arc, Mutex, RwLock};
use std::time::Instant;
use std::{io, thread};

use config::CStruct;
use config::ConfigEnum;
use config::State;
use config::{CBool, CInteger, CKwarg, CPath, CString, CType, Config};
use druid::im::{vector, Vector};
use druid::lens::{self, InArc, LensExt};
use druid::text::{Formatter, ParseFormatter, Selection, Validation, ValidationError};
use druid::widget::{
    Button, Checkbox, Controller, CrossAxisAlignment, Either, Flex, Label, LineBreaking, List,
    Maybe, Padding, Scroll, SizedBox, Spinner, Switch, TextBox, ViewSwitcher,
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
use futures::StreamExt;
use log::{debug, error, info, log_enabled, Level};
use serde::Serialize;
use tokio::time;
use tokio::time::Duration;

use crate::controller::{
    EditController, MainController, Msg, SettingController, TemplateController, MSG_THREAD,
    OPEN_EDIT,
};
use crate::cstruct_window::{c_option_window, CStructBuffer};
use crate::data::template_info::TemplateInfoSelect;
use crate::data::AppData;
use crate::edit_window::edit_window;
use crate::template::communication::RawCommunication;
use crate::template::node_type::site::TaskMsg;
use crate::template::node_type::NodeTypeData;
use crate::template::nodes::node_data::NodeData;
use crate::template::nodes::root_data::RootNodeData;
use crate::template::widget_data::TemplateData;
use crate::template::{DownloadArgs, Extensions, Mode, Template};
use crate::widgets::file_watcher::FileWatcher;
use crate::widgets::header::Header;
use crate::widgets::history_tree::History;
use crate::widgets::split::Split;
use crate::widgets::tree::Tree;
use crate::widgets::widget_ext::WidgetExt as _;

pub fn make_menu(_: Option<WindowId>, data: &AppData, _: &Env) -> Menu<AppData> {
    let mut base = Menu::empty();

    let mut open_recent = Menu::new("Open Recent");
    for path in data.recent_templates.iter() {
        if let Some(file_name) = path.file_name() {
            let path_clone = path.clone();
            open_recent = open_recent.entry(
                MenuItem::new(file_name.to_string_lossy().to_string()).on_activate(
                    move |ctx, _data: &mut AppData, _env| {
                        ctx.submit_command(commands::OPEN_FILE.with(FileInfo {
                            path: (*path_clone).clone(),
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
            Menu::new("File")
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

    base.rebuild_on(|old_data, data, _env| old_data.recent_templates != data.recent_templates)
}
pub fn build_ui() -> impl Widget<AppData> {
    Flex::column()
        .cross_axis_alignment(CrossAxisAlignment::Start)
        .with_child(
            SizedBox::empty()
                .controller(TemplateController::new())
                .padding(0.),
        )
        .with_child(
            SizedBox::empty()
                .controller(SettingController::new())
                .padding(0.)
                .lens(AppData::settings_window),
        )
        .with_child(
            SizedBox::empty()
                .controller(EditController::new())
                .padding(0.)
                .lens(AppData::edit_window),
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
            .on_save(
                |split, _ctx, data: &AppData, _env| split.set_split_point(data.split_point),
                |split, _ctx, data: &mut AppData, _env| {
                    data.split_point = split.current_split_point()
                },
            )
            .expand_width(),
            1.,
        )
        .with_child(info_view_selector_ui().lens(AppData::template_info_select))
}

fn info_view_ui() -> impl Widget<AppData> {
    ViewSwitcher::new(
        |data: &AppData, _env| data.template_info_select,
        |selector, _data, _env| match selector {
            TemplateInfoSelect::General => info_general().boxed(),
            TemplateInfoSelect::Folder => info_folder().boxed(),
            TemplateInfoSelect::History => info_history().boxed(),
            TemplateInfoSelect::Nothing => SizedBox::empty().boxed(),
        },
    )
}

fn info_general() -> impl Widget<AppData> {
    Label::dynamic(|data: &AppData, _env| {
        let node = data.get_selected_node();
        format!("{:#?}", node)
    })
    .scroll()
}

fn info_folder() -> impl Widget<AppData> {
    FileWatcher::new(
        |data: &AppData| match (data.get_settings(), data.get_selected_node()) {
            (Some(settings), Some(node)) => node
                .path
                .as_ref()
                .map(|path| settings.download.save_path.join(path)),
            _ => None,
        },
    )
    .controller(FolderController)
}

struct FolderController;

impl Controller<AppData, FileWatcher<AppData>> for FolderController {
    fn event(
        &mut self,
        child: &mut FileWatcher<AppData>,
        ctx: &mut EventCtx,
        event: &Event,
        data: &mut AppData,
        env: &Env,
    ) {
        dbg!(event);
        if TemplateInfoSelect::Folder == data.template_info_select {
            dbg!("Folder")
        } else {
            dbg!("not Folder")
        };
        child.event(ctx, event, data, env)
    }
}

fn info_history() -> impl Widget<AppData> {
    History::new().expand()
}

fn info_view_selector_ui() -> impl Widget<TemplateInfoSelect> {
    Flex::row()
        .with_child(
            Button::new("General").on_click(|_ctx, data: &mut TemplateInfoSelect, _env| {
                *data = TemplateInfoSelect::General
            }),
        )
        .with_child(
            Button::new("Folder").on_click(|_ctx, data: &mut TemplateInfoSelect, _env| {
                *data = TemplateInfoSelect::Folder
            }),
        )
        .with_child(
            Button::new("History").on_click(|_ctx, data: &mut TemplateInfoSelect, _env| {
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
        .on_click(|ctx, _, _env| ctx.submit_command(commands::SHOW_PREFERENCES));

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