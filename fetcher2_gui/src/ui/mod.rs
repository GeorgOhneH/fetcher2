use druid::widget::{Button, Controller, CrossAxisAlignment, Flex, Label, SizedBox, WidgetWrapper};
use druid::{
    commands, menu, Color, Command, Env, Event, EventCtx, FileInfo, LensExt, Menu, MenuItem,
    SingleUse, SysMods, Target, Widget, WidgetExt, WindowId,
};

use crate::controller::{
    EditController, Msg, SettingController, TemplateController, MSG_THREAD, OPEN_EDIT,
};
use crate::data::template::TemplateData;
use crate::data::template_info::{TemplateInfo, TemplateInfoSelect};
use crate::data::AppData;
use crate::widgets::file_watcher::FileWatcher;
use crate::widgets::history_tree::History;
use crate::widgets::info_view::InfoView;
use crate::widgets::split::Split;
use crate::widgets::widget_ext::WidgetExt as _;

pub fn make_menu(_: Option<WindowId>, data: &AppData, _: &Env) -> Menu<AppData> {
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
    let base = {
        Menu::new(druid::LocalizedString::new(""))
            .entry(
                Menu::new(druid::LocalizedString::new("macos-menu-application-menu"))
                    .entry(menu::sys::mac::application::preferences())
                    .separator()
                    .entry(menu::sys::mac::application::hide())
                    .entry(menu::sys::mac::application::hide_others()),
            )
            .entry(
                Menu::new(druid::LocalizedString::new("common-menu-file-menu"))
                    .entry(menu::sys::mac::file::new_file())
                    .entry(menu::sys::mac::file::open_file())
                    .entry(open_recent)
                    .separator()
                    .entry(
                        MenuItem::new("Open Edit")
                            .command(OPEN_EDIT)
                            .hotkey(SysMods::Cmd, "e"),
                    ),
            )
    };
    #[cfg(any(target_os = "windows", target_os = "linux"))]
    let base = {
        Menu::empty().entry(
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
        )
    };

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
                |split, _ctx, data: &AppData, _env| {
                    split.set_split_point(data.split_point.unwrap_or(0.5))
                },
                |split, _ctx, data: &mut AppData, _env| {
                    data.split_point = Some(split.current_split_point())
                },
            )
            .expand_width(),
            1.,
        )
        .with_child(
            info_view_selector_ui().lens(AppData::template_info.then(TemplateInfo::selected)),
        )
}

fn info_view_ui() -> impl Widget<AppData> {
    InfoView::new(
        |data: &AppData, _env| match data.template_info.selected {
            TemplateInfoSelect::General => Some(0),
            TemplateInfoSelect::Folder => Some(1),
            TemplateInfoSelect::History => Some(2),
            TemplateInfoSelect::Nothing => None,
        },
        [
            info_general().boxed(),
            info_folder().boxed(),
            info_history().boxed(),
        ],
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
    .on_save(
        |widget, _ctx, data: &AppData, _env| {
            if let Ok(sizes) = data.template_info.folder.header_sizes.clone().try_into() {
                widget.wrapped_mut().set_header_size(sizes);
            }
        },
        |widget, _ctx, data: &mut AppData, _env| {
            data.template_info.folder.header_sizes = widget.wrapped().get_header_sizes().into()
        },
    )
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
        child.event(ctx, event, data, env)
    }
}

fn info_history() -> impl Widget<AppData> {
    History::new()
        .on_save(
            |widget, _ctx, data: &AppData, _env| {
                if let Ok(sizes) = data.template_info.history.header_sizes.clone().try_into() {
                    widget.set_header_size(sizes);
                }
            },
            |widget, _ctx, data: &mut AppData, _env| {
                data.template_info.history.header_sizes = widget.get_header_sizes().into()
            },
        )
        .expand()
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
