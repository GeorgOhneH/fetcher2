use std::marker::PhantomData;
use std::path::PathBuf;
use std::sync::Arc;

use config::CStruct;
use config::Config;
use config::{CEnum, ConfigEnum};
use druid::commands::{CLOSE_WINDOW, SAVE_FILE, SAVE_FILE_AS};
use druid::im::Vector;
use druid::widget::prelude::*;
use druid::widget::{Button, Controller, ControllerHost, Flex, Label};
use druid::{
    commands, lens, Command, FileDialogOptions, Menu, MenuItem, SingleUse, UnitPoint, WindowId,
};
use druid::{InternalEvent, LensExt};
use druid::{Lens, Point, Target, Widget, WidgetExt, WidgetPod, WindowConfig, WindowLevel};
use druid_widget_nursery::selectors;
use serde::{Deserialize, Serialize};

use crate::controller::{Msg, MSG_THREAD};
use crate::cstruct_window::{c_option_window, APPLY};
use crate::template::communication::RawCommunication;
use crate::template::node_type::{NodeTypeEditData, NodeTypeEditKindData};
use crate::template::nodes::node_edit_data::NodeEditData;
use crate::template::nodes::root_edit_data::RootNodeEditData;
use crate::template::widget_edit_data::TemplateEditData;
use crate::template::Template;
use crate::ui::AppData;
use crate::widgets::tree::{DataNodeIndex, NodeIndex, Tree};
use crate::data::win::WindowState;

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
    pub child: WidgetPod<EditWindowData, Box<dyn Widget<EditWindowData>>>,
    pub data: Option<EditWindowData>,
    new: bool,
}

impl DataBuffer {
    pub fn new(child: impl Widget<EditWindowData> + 'static, new: bool) -> Self {
        Self {
            child: WidgetPod::new(Box::new(child)),
            data: None,
            new,
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

    fn get_template(&self) -> &TemplateEditData {
        &self
            .data
            .as_ref()
            .expect("Called before widget Added event")
            .edit_template
    }
}

impl Widget<EditWindowData> for DataBuffer {
    fn event(&mut self, ctx: &mut EventCtx, event: &Event, data: &mut EditWindowData, env: &Env) {
        match event {
            Event::Command(command) if command.is(SAVE_EDIT) => {
                ctx.set_handled();
                if let Some(save_path) = &self.get_template().save_path {
                    Self::send_update_msg(ctx, self.get_template().root.clone(), save_path.clone());
                    *data = self.data.as_ref().unwrap().clone();
                    ctx.submit_command(CLOSE_WINDOW);
                }
            }
            Event::Command(command) if command.is(SAVE_FILE_AS) => {
                ctx.set_handled();
                let save_path = command.get_unchecked(SAVE_FILE_AS);
                Self::send_update_msg(
                    ctx,
                    self.get_template().root.clone(),
                    save_path.path.clone(),
                );
                *data = self.data.as_ref().unwrap().clone();
                ctx.submit_command(CLOSE_WINDOW);
            }
            _ => (),
        }

        let old_data = self.data.clone();
        self.child
            .event(ctx, event, self.data.as_mut().unwrap(), env);
        if !old_data.same(&self.data) {
            dbg!("CHANGEF");
            ctx.request_update()
        }
    }

    fn lifecycle(
        &mut self,
        ctx: &mut LifeCycleCtx,
        event: &LifeCycle,
        data: &EditWindowData,
        env: &Env,
    ) {
        if let LifeCycle::WidgetAdded = event {
            let new_data = if self.new {
                let mut n_data = data.clone();
                n_data.edit_template = TemplateEditData::new();
                n_data
            } else {
                data.clone()
            };
            self.data = Some(new_data)
        }
        self.child
            .lifecycle(ctx, event, self.data.as_ref().unwrap(), env)
    }

    fn update(
        &mut self,
        ctx: &mut UpdateCtx,
        old_data: &EditWindowData,
        data: &EditWindowData,
        env: &Env,
    ) {
        self.child.update(ctx, self.data.as_ref().unwrap(), env)
    }

    fn layout(
        &mut self,
        ctx: &mut LayoutCtx,
        bc: &BoxConstraints,
        data: &EditWindowData,
        env: &Env,
    ) -> Size {
        let size = self.child.layout(ctx, bc, self.data.as_ref().unwrap(), env);
        self.child
            .set_origin(ctx, self.data.as_ref().unwrap(), env, Point::ORIGIN);
        size
    }

    fn paint(&mut self, ctx: &mut PaintCtx, data: &EditWindowData, env: &Env) {
        self.child.paint(ctx, self.data.as_ref().unwrap(), env)
    }
}

pub struct NodeController {}

impl NodeController {
    pub fn new() -> Self {
        Self {}
    }
}

impl<L, S, const N: usize>
    Controller<RootNodeEditData, Tree<RootNodeEditData, NodeEditData, L, S, N>> for NodeController
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
            Event::WindowConnected => {
                if data.children.len() == 0 {
                    data.children.push_back(NodeEditData::new(true))
                }
            }
            Event::MouseDown(ref mouse) if mouse.button.is_right() => {
                if let Some(idx) = child.node_at(mouse.pos) {
                    ctx.show_context_menu(make_node_menu(idx), mouse.window_pos);
                    return;
                }
            }
            Event::Command(cmd) if cmd.is(DELETE_NODE) => {
                ctx.set_handled();
                let idx = cmd.get_unchecked(DELETE_NODE);
                data.remove(idx);
                if data.children.len() == 0 {
                    data.children.push_back(NodeEditData::new(true))
                }
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

struct NodeWindowController;

impl<W: Widget<EditWindowData>> Controller<EditWindowData, W> for NodeWindowController {
    fn event(
        &mut self,
        child: &mut W,
        ctx: &mut EventCtx,
        event: &Event,
        data: &mut EditWindowData,
        env: &Env,
    ) {
        match event {
            Event::Command(cmd) if cmd.is(OPEN_NODE) => {
                ctx.set_handled();
                let idx = cmd.get_unchecked(OPEN_NODE);

                let (size, pos) = if let Some(win_state) = &data.node_win_state {
                    (win_state.get_size(), win_state.get_pos())
                } else {
                    WindowState::default_size_pos(ctx.window())
                };
                ctx.new_sub_window(
                    WindowConfig::default()
                        .show_titlebar(true)
                        .window_size(size)
                        .set_position(pos)
                        .set_level(WindowLevel::Modal),
                    node_window(idx),
                    data.clone(),
                    env.clone(),
                );
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

pub fn edit_window(new: bool) -> impl Widget<EditWindowData> {
    DataBuffer::new(_edit_window(), new)
}

fn _edit_window() -> impl Widget<EditWindowData> {
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
    .controller(NodeController::new())
    .lens(EditWindowData::edit_template.then(TemplateEditData::root))
    .controller(NodeWindowController);
    Flex::column()
        .with_flex_child(tree.expand(), 1.0)
        .with_child(
            Flex::row()
                .with_child(
                    Button::new("Save")
                        .on_click(|ctx, data: &mut EditWindowData, env| {
                            ctx.submit_command(SAVE_EDIT.to(Target::Window(ctx.window_id())));
                        })
                        .disabled_if(|data: &EditWindowData, env| {
                            data.edit_template.save_path.is_none()
                        }),
                )
                .with_child(Button::new("Save as").on_click(
                    |ctx, data: &mut EditWindowData, env| {
                        ctx.submit_command(
                            commands::SHOW_SAVE_PANEL.with(FileDialogOptions::default()),
                        );
                    },
                ))
                .with_child(Button::new("Cancel").on_click(
                    |ctx, data: &mut EditWindowData, env| {
                        ctx.submit_command(CLOSE_WINDOW);
                    },
                )),
        )
}

fn node_window(idx: &NodeIndex) -> impl Widget<EditWindowData> {
    c_option_window(
        Some("Node"),
        Some(Box::new(|ctx, old_data, data: &mut NodeTypeEditData, _| {
            if let Some(old) = old_data {
                if !old.same(data) {
                    data.invalidate_cache();
                }
            }
        })),
    )
    .lens(
        EditWindowData::edit_template
            .then(TemplateEditData::root.then(NodeLens::new(idx.clone()).then(NodeEditData::ty))),
    )
    .controller(NodeWindowSaveController)
}

struct NodeWindowSaveController;

impl<W: Widget<EditWindowData>> Controller<EditWindowData, W> for NodeWindowSaveController {
    fn event(
        &mut self,
        child: &mut W,
        ctx: &mut EventCtx,
        event: &Event,
        data: &mut EditWindowData,
        env: &Env,
    ) {
        if let Event::WindowCloseRequested = event {
            data.node_win_state = Some(WindowState::from_win(ctx.window()));
        }
        child.event(ctx, event, data, env)
    }
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
