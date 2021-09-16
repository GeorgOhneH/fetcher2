use druid::widget::prelude::*;
use druid::widget::ControllerHost;
use druid::{Selector, WidgetExt as _};

use crate::widgets::on_command::OnCmd;

pub trait WidgetExt<T: Data>: Widget<T> + Sized + 'static {
    fn on_command2<CT: 'static>(
        self,
        selector: Selector<CT>,
        handler: impl Fn(&mut EventCtx, &CT, &mut T, &Env) + 'static,
    ) -> ControllerHost<Self, OnCmd<CT, T>> {
        self.controller(OnCmd::new(selector, handler))
    }
}

impl<T: Data, W: Widget<T> + 'static> WidgetExt<T> for W {}
