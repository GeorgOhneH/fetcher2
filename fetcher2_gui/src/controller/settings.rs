use druid::{commands, SingleUse, WidgetExt, WindowConfig, WindowLevel};
use druid::{Env, Event, EventCtx, LifeCycle, LifeCycleCtx, Widget};
use druid::widget::Controller;

use crate::controller::{Msg, MSG_THREAD};
use crate::cstruct_window::c_option_window;
use crate::data::settings::{OptionSettings, Settings};
use crate::data::win::SubWindowInfo;
use crate::widgets::sub_window_widget::SubWindow;

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
                move |inner_ctx: &mut EventCtx, _old_data, data: &mut Settings, _env| {
                    inner_ctx.submit_command(
                        MSG_THREAD
                            .with(SingleUse::new(Msg::NewSettings(data.download.clone())))
                            .to(main_win_id),
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
                .set_level(WindowLevel::Modal(ctx.window().clone())),
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
