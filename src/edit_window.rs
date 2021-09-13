use crate::cstruct_window::{c_option_window, APPLY};
use crate::delegate::{Msg, MSG_THREAD};
use crate::template::node_type::{NodeTypeEditData, NodeTypeEditKindData};
use crate::template::nodes::node_edit_data::NodeEditData;
use crate::template::nodes::root_edit_data::RootNodeEditData;
use crate::template::widget_edit_data::TemplateEditData;
use crate::widgets::tree::{NodeIndex, Tree};
use config::CStruct;
use config::Config;
use config::{CEnum, ConfigEnum};
use druid::commands::CLOSE_WINDOW;
use druid::widget::prelude::*;
use druid::widget::{Button, Flex, Label};
use druid::{lens, Command, SingleUse, WindowId};
use druid::{InternalEvent, LensExt};
use druid::{Lens, Point, Target, Widget, WidgetExt, WidgetPod, WindowConfig, WindowLevel};
use druid_widget_nursery::selectors;
use std::marker::PhantomData;
use std::sync::Arc;

selectors! {
    SAVE_EDIT
}

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
        match event {
            Event::Command(command) if command.is(SAVE_EDIT) => {
                ctx.set_handled();
                ctx.submit_command(Command::new(
                    MSG_THREAD,
                    SingleUse::new(Msg::UpdateEditData(self.edit_data.clone())),
                    Target::Global,
                ));
                ctx.window().close();
            }
            _ => (),
        }

        let old_data = self.edit_data.clone();
        self.child.event(ctx, event, &mut self.edit_data, env);
        if !old_data.same(&self.edit_data) {
            dbg!("DATA CHAGEd");
            ctx.request_update()
        }
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
    let tree = Tree::new(
        [Label::new("Hello")],
        [Arc::new(|| {
            Label::dynamic(|data: &NodeEditData, _env| data.name()).boxed()
        })],
        NodeEditData::expanded,
        RootNodeEditData::selected,
    )
    .on_activate(|ctx, data: &mut RootNodeEditData, env, idx| {
        let window = ctx.window();
        let win_pos = window.get_position();
        let (win_size_w, win_size_h) = window.get_size().into();
        let (size_w, size_h) = (f64::min(600., win_size_w), f64::min(600., win_size_h));
        let pos = win_pos + ((win_size_w - size_w) / 2., (win_size_h - size_h) / 2.);
        ctx.new_sub_window(
            WindowConfig::default()
                .show_titlebar(true)
                .window_size(Size::new(size_w, size_h))
                .set_position(pos)
                .set_level(WindowLevel::Modal),
            node_window(idx),
            data.clone(),
            env.clone(),
        );
    })
    .padding(0.)
    .lens(TemplateEditData::root);

    Flex::column()
        .with_flex_child(tree.expand(), 1.0)
        .with_child(
            Flex::row().with_child(
                Button::new("Save")
                    .on_click(|ctx, data: &mut TemplateEditData, env| {
                        ctx.submit_command(SAVE_EDIT.to(Target::Window(ctx.window_id())));
                    })
                // .disabled_if(|data: &Option<T>, env| match &data.c_struct {
                //     Some(c_struct) => !matches!(c_struct.state(), State::Valid),
                //     None => true,
                // }),
            ).with_child(
                Button::new("Cancel")
                    .on_click(|ctx, data: &mut TemplateEditData, env| {
                        ctx.window().close();
                    })),
        )
}

fn node_window(idx: &NodeIndex) -> impl Widget<RootNodeEditData> {
    c_option_window().lens(NodeLens::new(idx.clone()).then(NodeEditData::ty))
}

struct NodeLens {
    idx: NodeIndex,
}

impl NodeLens {
    pub fn new(idx: NodeIndex) -> Self {
        Self { idx }
    }
}

impl Lens<RootNodeEditData, NodeEditData> for NodeLens {
    fn with<V, F: FnOnce(&NodeEditData) -> V>(&self, data: &RootNodeEditData, f: F) -> V {
        f(data.node(&self.idx))
    }

    fn with_mut<V, F: FnOnce(&mut NodeEditData) -> V>(
        &self,
        data: &mut RootNodeEditData,
        f: F,
    ) -> V {
        let mut new_node = data.node(&self.idx).to_owned();
        let v = f(&mut new_node);
        if !new_node.same(data.node(&self.idx)) {
            *data.node_mut(&self.idx) = new_node;
        }
        v
    }
}
