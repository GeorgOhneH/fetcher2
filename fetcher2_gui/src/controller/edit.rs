use druid::{commands, WindowConfig, WindowLevel};
use druid::{Env, Event, EventCtx, Widget};
use druid::widget::Controller;
use druid_widget_nursery::selectors;

use crate::background_thread::NEW_EDIT_TEMPLATE;
use crate::data::edit::EditWindowData;
use crate::data::win::SubWindowInfo;
use crate::edit_window::edit_window;
use crate::widgets::sub_window_widget::SubWindow;

selectors! {
    OPEN_EDIT
}

pub struct EditController {}

impl EditController {
    pub fn new() -> Self {
        Self {}
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
                .set_level(WindowLevel::Modal(ctx.window().clone())),
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
                let (edit_root, path) = cmd.get_unchecked(NEW_EDIT_TEMPLATE).take().unwrap();
                data.data.edit_template.root = edit_root;
                data.data.edit_template.save_path = path;
                return;
            }
            Event::Command(cmd) if cmd.is(OPEN_EDIT) => {
                ctx.set_handled();
                self.make_sub_window(ctx, env, data, false);
                return;
            }
            Event::Command(cmd) if cmd.is(commands::NEW_FILE) => {
                ctx.set_handled();
                self.make_sub_window(ctx, env, data, true);
                return;
            }
            _ => (),
        }
        child.event(ctx, event, data, env)
    }
}
