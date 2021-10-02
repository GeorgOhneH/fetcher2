use druid::widget::prelude::*;
use druid::widget::ControllerHost;
use druid::{Selector, WidgetExt as _};

use crate::widgets::on_save::Save;

pub trait WidgetExt<T: Data>: Widget<T> + Sized + 'static {
    fn on_save(
        self,
        init: impl Fn(&mut Self, &mut LifeCycleCtx, &T, &Env) + 'static,
        save: impl Fn(&mut Self, &mut EventCtx, &mut T, &Env) + 'static,
    ) -> ControllerHost<Self, Save<T, Self>> {
        self.controller(Save::new(init, save))
    }
}

impl<T: Data, W: Widget<T> + 'static> WidgetExt<T> for W {}
