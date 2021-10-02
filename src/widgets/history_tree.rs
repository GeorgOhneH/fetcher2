use std::fs::DirEntry;
use std::path::{Path, PathBuf};
use std::sync::mpsc::channel;
use std::sync::Arc;
use std::thread;
use std::time::Duration;
use std::{fs, io};

use crossbeam_channel::{Receiver, Select, Sender};
use druid::im::Vector;
use druid::widget::Label;
use druid::{
    BoxConstraints, Data, Env, Event, EventCtx, ExtEventSink, LayoutCtx, Lens, LifeCycle,
    LifeCycleCtx, PaintCtx, Point, SingleUse, Size, Target, UpdateCtx, Widget, WidgetExt, WidgetId,
    WidgetPod,
};
use druid_widget_nursery::{selectors, WidgetExt as _};
use futures::SinkExt;
use notify::{recommended_watcher, RecommendedWatcher, RecursiveMode, Watcher};

use crate::data::AppData;
use crate::template::node_type::site::{MsgKind, TaskMsg};
use crate::widgets::tree::node::{impl_simple_tree_node, TreeNode};
use crate::widgets::tree::root::{impl_simple_tree_root, TreeNodeRoot};
use crate::widgets::tree::{DataNodeIndex, Tree};
use crate::Result;
use std::fmt::{Display, Formatter};

#[derive(Data, Clone, Debug, PartialEq)]
pub enum Type {
    AddedFile,
    ReplacedFile,
    NotModified,
    FileChecksumSame,
    AlreadyExist,
    ForbiddenExtension(Option<String>),

    InnerReplaced,
}

impl Display for Type {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let str = match self {
            Self::AddedFile => "New File",
            Self::ReplacedFile => "Replaced File",
            Self::NotModified => "Not Modified",
            Self::FileChecksumSame => "Same File Already on Disk",
            Self::AlreadyExist => "Cached Checksum didn't Change",
            Self::ForbiddenExtension(_) => "Extension is Forbidden",
            Self::InnerReplaced => "Old File",
        };
        f.write_str(str)
    }
}

impl Type {
    fn from_msg(msg_kind: MsgKind, parent_path: &str) -> (Self, Vector<Entry>) {
        match msg_kind {
            MsgKind::AddedFile => (Type::AddedFile, Vector::new()),
            MsgKind::ReplacedFile(path) => (
                Type::ReplacedFile,
                vec![Entry::inner_replaced(path, parent_path.to_owned())].into(),
            ),
            MsgKind::NotModified => (Type::NotModified, Vector::new()),
            MsgKind::FileChecksumSame => (Type::FileChecksumSame, Vector::new()),
            MsgKind::AlreadyExist => (Type::AlreadyExist, Vector::new()),
            MsgKind::ForbiddenExtension(extension) => {
                (Type::ForbiddenExtension(extension), Vector::new())
            }
        }
    }
}

#[derive(Data, Clone, Debug, Lens)]
pub struct Entry {
    children: Vector<Entry>,
    ty: Type,
    name: String,
    parent_path: String,
    #[data(eq)]
    full_path: PathBuf,
    expanded: bool,
}

impl_simple_tree_node! {Entry}

impl Entry {
    pub fn new(task_msg: TaskMsg) -> Self {
        let rel_path = task_msg
            .rel_path
            .parent()
            .map(|path| String::from(std::path::MAIN_SEPARATOR) + path.to_string_lossy().as_ref())
            .unwrap_or_else(|| String::from(std::path::MAIN_SEPARATOR));
        let (ty, children) = Type::from_msg(task_msg.kind, &rel_path);
        let name = task_msg
            .full_path
            .file_name()
            .map(|os_str| os_str.to_string_lossy().to_string())
            .unwrap_or_else(|| "".to_owned());
        Self {
            expanded: false,
            full_path: task_msg.full_path,
            parent_path: rel_path,
            ty,
            name,
            children,
        }
    }

    fn inner_replaced(path: PathBuf, parent_path: String) -> Self {
        let name = path
            .file_name()
            .map(|os_str| os_str.to_string_lossy().to_string())
            .unwrap_or_else(|| "".to_owned());
        Self {
            expanded: false,
            name,
            full_path: path,
            parent_path,
            ty: Type::InnerReplaced,
            children: Vector::new(),
        }
    }
}

#[derive(Data, Clone, Debug, Lens)]
pub struct EntryRoot {
    children: Vector<Entry>,
    selected: Vector<DataNodeIndex>,
}

impl_simple_tree_root! {EntryRoot, Entry}

impl EntryRoot {
    pub fn empty() -> Self {
        Self {
            children: Vector::new(),
            selected: Vector::new(),
        }
    }

    pub fn new(history: Vector<TaskMsg>) -> Self {
        let children = history
            .iter()
            .rev()
            .take(100)
            .map(|task_msg| Entry::new(task_msg.clone()))
            .collect();

        Self {
            children,
            selected: Vector::new(),
        }
    }
}

pub struct History {
    tree: WidgetPod<
        EntryRoot,
        Tree<
            EntryRoot,
            Entry,
            entry_derived_lenses::expanded,
            entry_root_derived_lenses::selected,
            3,
        >,
    >,
    root: EntryRoot,
}

impl History {
    pub fn new() -> Self {
        let tree = Tree::new(
            [Label::new("Name"), Label::new("Path"), Label::new("Note")],
            [
                Arc::new(|| Label::dynamic(|data: &Entry, _env| data.name.clone()).boxed()),
                Arc::new(|| Label::dynamic(|data: &Entry, _env| data.parent_path.clone()).boxed()),
                Arc::new(|| Label::dynamic(|data: &Entry, _env| data.ty.to_string()).boxed()),
            ],
            Entry::expanded,
            EntryRoot::selected,
        )
        .sizes([400., 400., 400.])
        .on_activate(|_ctx, root, _env, idx| {
            let node = root.node(idx);
            open::that_in_background(&node.full_path);
        });
        Self {
            tree: WidgetPod::new(tree),
            root: EntryRoot::empty(),
        }
    }
}

impl Widget<AppData> for History {
    fn event(&mut self, ctx: &mut EventCtx, event: &Event, _: &mut AppData, env: &Env) {
        self.tree.event(ctx, event, &mut self.root, env)
    }

    fn lifecycle(&mut self, ctx: &mut LifeCycleCtx, event: &LifeCycle, data: &AppData, env: &Env) {
        if let LifeCycle::WidgetAdded = event {
            if let Some(history) = data.get_selected_history() {
                self.root = EntryRoot::new(history)
            }
        };
        self.tree.lifecycle(ctx, event, &self.root, env)
    }

    fn update(&mut self, ctx: &mut UpdateCtx, old_data: &AppData, data: &AppData, env: &Env) {
        let maybe_history = data.get_selected_history();
        if !old_data.get_selected_history().same(&maybe_history) {
            self.root = match maybe_history {
                Some(history) => EntryRoot::new(history),
                None => EntryRoot::empty(),
            }
        }
        self.tree.update(ctx, &self.root, env);
    }

    fn layout(&mut self, ctx: &mut LayoutCtx, bc: &BoxConstraints, _: &AppData, env: &Env) -> Size {
        let size = self.tree.layout(ctx, bc, &self.root, env);
        self.tree.set_origin(ctx, &self.root, env, Point::ORIGIN);
        size
    }

    fn paint(&mut self, ctx: &mut PaintCtx, _: &AppData, env: &Env) {
        self.tree.paint(ctx, &self.root, env)
    }
}
