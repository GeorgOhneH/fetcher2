use std::future::Future;
use std::ops::Deref;
use std::pin::Pin;
use std::task::{Context, Poll};

use druid::widget::{Flex, Label, LineBreaking};
use druid::{Env, EventCtx, Widget, WidgetExt, WindowConfig, WindowLevel};
use futures::FutureExt;
use tokio::task::{JoinError, JoinHandle};

use crate::TError;

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
