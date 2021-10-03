use druid::widget::{Flex, Label, LineBreaking};
use druid::{Env, EventCtx, Widget, WidgetExt, WindowConfig, WindowLevel};

use crate::data::win::WindowState;
use crate::data::AppData;
use crate::TError;

pub fn show_err(ctx: &mut EventCtx, data: &AppData, env: &Env, err: TError, title: &str) {
    let (size, pos) = WindowState::default_size_pos(ctx.window());
    ctx.new_sub_window(
        WindowConfig::default()
            .show_titlebar(true)
            .set_position(pos)
            .window_size(size)
            .set_level(WindowLevel::Modal),
        err_widget(err, title),
        data.clone(),
        env.clone(),
    );
}

fn err_widget(err: TError, title: &str) -> impl Widget<AppData> {
    Flex::column()
        .with_child(Label::new(title))
        .with_child(
            Label::new(format!("{:?}", err.kind)).with_line_break_mode(LineBreaking::WordWrap),
        )
        .with_flex_child(
            Label::new(format!("{}", err.backtrace))
                .with_line_break_mode(LineBreaking::Overflow)
                .scroll(),
            1.0,
        )
}
