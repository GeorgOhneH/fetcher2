use crate::template::nodes::node_edit_data::NodeEditData;
use crate::template::nodes::root_edit_data::RootNodeEditData;
use crate::template::widget_edit_data::TemplateEditData;
use crate::widgets::tree::Tree;
use druid::widget::prelude::*;
use druid::widget::Label;
use druid::{Point, Widget, WidgetExt, WidgetPod};
use std::marker::PhantomData;
use std::sync::Arc;

pub struct DataBuffer {
    pub child: WidgetPod<TemplateEditData, Box<dyn Widget<TemplateEditData>>>,
    pub edit_data: TemplateEditData,
}

impl DataBuffer {
    pub fn new(
        child: impl Widget<TemplateEditData> + 'static,
        edit_data: TemplateEditData,
    ) -> Self {
        Self {
            child: WidgetPod::new(child.boxed()),
            edit_data,
        }
    }
}

impl<T: Data> Widget<T> for DataBuffer {
    fn event(&mut self, ctx: &mut EventCtx, event: &Event, data: &mut T, env: &Env) {
        self.child.event(ctx, event, &mut self.edit_data, env)
    }

    fn lifecycle(&mut self, ctx: &mut LifeCycleCtx, event: &LifeCycle, data: &T, env: &Env) {
        self.child.lifecycle(ctx, event, &self.edit_data, env)
    }

    fn update(&mut self, ctx: &mut UpdateCtx, old_data: &T, data: &T, env: &Env) {
        self.child.update(ctx, &self.edit_data, env)
    }

    fn layout(&mut self, ctx: &mut LayoutCtx, bc: &BoxConstraints, data: &T, env: &Env) -> Size {
        let size = self.child.layout(ctx, bc, &self.edit_data, env);
        self.child
            .set_origin(ctx, &self.edit_data, env, Point::ORIGIN);
        size
    }

    fn paint(&mut self, ctx: &mut PaintCtx, data: &T, env: &Env) {
        self.child.paint(ctx, &self.edit_data, env)
    }
}

pub fn edit_window(data: TemplateEditData) -> impl Widget<()> {
    DataBuffer::new(_edit_window(), data)
}

fn _edit_window() -> impl Widget<TemplateEditData> {
    Tree::new(
        [Label::new("Hello")],
        [Arc::new(|| {
            Label::dynamic(|data: &NodeEditData, _env| data.ty.name()).boxed()
        })],
        NodeEditData::expanded,
        RootNodeEditData::selected,
    )
    .lens(TemplateEditData::root)
}
