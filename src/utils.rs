use crate::controller::SubWindowInfo;
use crate::TError;
use druid::widget::{Flex, Label, LineBreaking};
use druid::{Env, EventCtx, Widget, WidgetExt, WindowConfig, WindowLevel};
use futures::FutureExt;
use std::future::Future;
use std::ops::Deref;
use std::pin::Pin;
use std::task::{Context, Poll};
use tokio;
use tokio::task::{JoinError, JoinHandle};

pub struct JoinHandleDrop<T>(JoinHandle<T>);

impl<T> Drop for JoinHandleDrop<T> {
    fn drop(&mut self) {
        self.0.abort()
    }
}

impl<T> Future for JoinHandleDrop<T> {
    type Output = std::result::Result<T, JoinError>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        self.0.poll_unpin(cx)
    }
}

pub fn spawn_drop<T>(future: T) -> JoinHandleDrop<T::Output>
where
    T: Future + Send + 'static,
    T::Output: Send + 'static,
{
    JoinHandleDrop(tokio::spawn(future))
}

pub fn show_err(ctx: &mut EventCtx, env: &Env, err: TError, title: &str) {
    let (size, pos) = SubWindowInfo::<()>::size_pos(ctx.window());
    ctx.new_sub_window(
        WindowConfig::default()
            .show_titlebar(true)
            .set_position(pos)
            .window_size(size)
            .set_level(WindowLevel::Modal),
        err_widget(err, title),
        (),
        env.clone(),
    );
}

fn err_widget(err: TError, title: &str) -> impl Widget<()> {
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
