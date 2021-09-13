use crate::cstruct_window::{c_option_window, APPLY};
use crate::delegate::{Msg, MSG_THREAD};
use crate::template::communication::RawCommunication;
use crate::template::node_type::{NodeTypeEditData, NodeTypeEditKindData};
use crate::template::nodes::node_edit_data::NodeEditData;
use crate::template::nodes::root_edit_data::RootNodeEditData;
use crate::template::widget_edit_data::TemplateEditData;
use crate::template::Template;
use crate::ui::AppData;
use crate::widgets::tree::{DataNodeIndex, NodeIndex, Tree};
use config::CStruct;
use config::Config;
use config::{CEnum, ConfigEnum};
use druid::commands::{CLOSE_WINDOW, SAVE_FILE, SAVE_FILE_AS};
use druid::im::Vector;
use druid::widget::prelude::*;
use druid::widget::{Button, Controller, Flex, Label};
use druid::{
    commands, lens, Command, FileDialogOptions, Menu, MenuItem, SingleUse, UnitPoint, WindowId,
};
use druid::{InternalEvent, LensExt};
use druid::{Lens, Point, Target, Widget, WidgetExt, WidgetPod, WindowConfig, WindowLevel};
use druid_widget_nursery::selectors;
use std::marker::PhantomData;
use std::path::PathBuf;
use std::sync::Arc;

selectors! {
    SAVE_EDIT,
}

selectors! {
    OPEN_NODE: NodeIndex,
    DELETE_NODE: NodeIndex,
    ADD_NODE: (NodeIndex, NodePosition),
}

#[derive(Copy, Clone, Debug)]
pub enum NodePosition {
    Above,
    Below,
    Child,
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

    fn send_update_msg(ctx: &mut EventCtx, root: RootNodeEditData, save_path: PathBuf) {
        let comm = RawCommunication::new(ctx.get_external_handle());
        let template = Template::from_raw(root, comm, save_path);

        ctx.submit_command(Command::new(
            MSG_THREAD,
            SingleUse::new(Msg::NewTemplate(template)),
            Target::Global,
        ));
    }
}

impl<T: Data> Widget<T> for DataBuffer {
    fn event(&mut self, ctx: &mut EventCtx, event: &Event, data: &mut T, env: &Env) {
        match event {
            Event::Command(command) if command.is(SAVE_EDIT) => {
                ctx.set_handled();
                if let Some(save_path) = &self.edit_data.save_path {
                    Self::send_update_msg(ctx, self.edit_data.root.clone(), save_path.clone());
                    ctx.window().close();
                }
            }
            Event::Command(command) if command.is(SAVE_FILE_AS) => {
                ctx.set_handled();
                let save_path = command.get_unchecked(SAVE_FILE_AS);
                Self::send_update_msg(ctx, self.edit_data.root.clone(), save_path.path.clone());
                ctx.window().close();
            }
            _ => (),
        }

        let old_data = self.edit_data.clone();
        self.child.event(ctx, event, &mut self.edit_data, env);
        if !old_data.same(&self.edit_data) {
            dbg!("CHANGEF");
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

pub struct EditController;

impl<L, S, const N: usize>
    Controller<RootNodeEditData, Tree<RootNodeEditData, NodeEditData, L, S, N>> for EditController
where
    L: Lens<NodeEditData, bool> + Clone + 'static,
    S: Lens<RootNodeEditData, Vector<DataNodeIndex>> + Clone + 'static,
{
    fn event(
        &mut self,
        child: &mut Tree<RootNodeEditData, NodeEditData, L, S, N>,
        ctx: &mut EventCtx,
        event: &Event,
        data: &mut RootNodeEditData,
        env: &Env,
    ) {
        match event {
            Event::MouseDown(ref mouse) if mouse.button.is_right() => {
                if let Some(idx) = child.node_at(mouse.pos) {
                    ctx.show_context_menu(make_node_menu(idx), mouse.window_pos);
                    return;
                }
            }
            Event::Command(cmd) if cmd.is(OPEN_NODE) => {
                ctx.set_handled();
                let idx = cmd.get_unchecked(OPEN_NODE);
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
                return;
            }
            Event::Command(cmd) if cmd.is(DELETE_NODE) => {
                ctx.set_handled();
                let idx = cmd.get_unchecked(DELETE_NODE);
                data.remove(idx);
                ctx.request_update();
                ctx.request_paint();
                return;
            }
            Event::Command(cmd) if cmd.is(ADD_NODE) => {
                ctx.set_handled();
                let (idx, pos) = cmd.get_unchecked(ADD_NODE);
                data.insert_node(idx, *pos);
                ctx.request_update();
                ctx.request_paint();
                return;
            }
            _ => (),
        }
        child.event(ctx, event, data, env)
    }
}

fn make_node_menu(idx: NodeIndex) -> Menu<AppData> {
    let idx1 = idx.clone();
    let idx2 = idx.clone();
    let idx3 = idx.clone();
    let idx4 = idx.clone();
    let idx5 = idx.clone();
    Menu::empty()
        .entry(
            MenuItem::new("Edit").on_activate(move |ctx, data: &mut AppData, _env| {
                ctx.submit_command(OPEN_NODE.with(idx1.clone()))
            }),
        )
        .entry(
            MenuItem::new("Delete").on_activate(move |ctx, data: &mut AppData, _env| {
                ctx.submit_command(DELETE_NODE.with(idx2.clone()))
            }),
        )
        .separator()
        .entry(MenuItem::new("Add new node above").on_activate(
            move |ctx, data: &mut AppData, _env| {
                ctx.submit_command(ADD_NODE.with((idx3.clone(), NodePosition::Above)))
            },
        ))
        .entry(MenuItem::new("Add new node below").on_activate(
            move |ctx, data: &mut AppData, _env| {
                ctx.submit_command(ADD_NODE.with((idx4.clone(), NodePosition::Below)))
            },
        ))
        .entry(MenuItem::new("Add new node as child").on_activate(
            move |ctx, data: &mut AppData, _env| {
                ctx.submit_command(ADD_NODE.with((idx5.clone(), NodePosition::Child)))
            },
        ))
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
        ctx.submit_command(OPEN_NODE.with(idx.clone()));
    })
    .controller(EditController {})
    .padding(0.)
    .lens(TemplateEditData::root);

    Flex::column()
        .with_flex_child(tree.expand(), 1.0)
        .with_child(
            Flex::row()
                .with_child(
                    Button::new("Save")
                        .on_click(|ctx, data: &mut TemplateEditData, env| {
                            ctx.submit_command(SAVE_EDIT.to(Target::Window(ctx.window_id())));
                        })
                        .disabled_if(|data: &TemplateEditData, env| data.save_path.is_none()),
                )
                .with_child(Button::new("Save as").on_click(
                    |ctx, data: &mut TemplateEditData, env| {
                        ctx.submit_command(
                            commands::SHOW_SAVE_PANEL.with(FileDialogOptions::default()),
                        );
                    },
                ))
                .with_child(Button::new("Cancel").on_click(
                    |ctx, data: &mut TemplateEditData, env| {
                        ctx.window().close();
                    },
                )),
        )
}

fn node_window(idx: &NodeIndex) -> impl Widget<RootNodeEditData> {
    c_option_window(Some(Box::new(
        |ctx, old_data, data: &mut NodeTypeEditData| {
            if let Some(old) = old_data {
                if !old.same(data) {
                    data.invalidate_cache();
                }
            }
        },
    )))
    .lens(NodeLens::new(idx.clone()).then(NodeEditData::ty))
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
