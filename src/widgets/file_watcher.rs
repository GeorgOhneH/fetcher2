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

use crate::widgets::tree::node::{TreeNode, impl_simple_tree_node};
use crate::widgets::tree::root::{TreeNodeRoot, impl_simple_tree_root};
use crate::widgets::tree::{DataNodeIndex, Tree};
use crate::Result;

selectors! {
    NEW_ROOT: SingleUse<io::Result<EntryRoot>>
}

#[derive(Data, Clone, Debug, PartialEq)]
pub enum Type {
    Folder,
    File,
}

#[derive(Data, Clone, Debug, Lens)]
pub struct Entry {
    name: String,
    children: Vector<Entry>,
    ty: Type,
    expanded: bool,
}

impl_simple_tree_node!{Entry}

impl Entry {
    pub fn new(entry: fs::DirEntry) -> io::Result<Self> {
        let meta_data = entry.metadata()?;

        if meta_data.is_file() {
            Ok(Self {
                children: Vector::new(),
                expanded: false,
                name: entry.file_name().to_string_lossy().to_string(),
                ty: Type::File,
            })
        } else {
            let children = fs::read_dir(entry.path())?
                .map(|entry| Entry::new(entry?))
                .collect::<io::Result<Vector<_>>>()?;
            Ok(Self {
                children,
                expanded: false,
                name: entry.file_name().to_string_lossy().to_string(),
                ty: Type::Folder,
            })
        }
    }
}


#[derive(Data, Clone, Debug, Lens)]
pub struct EntryRoot {
    children: Vector<Entry>,
    selected: Vector<DataNodeIndex>,
}

impl_simple_tree_root!{EntryRoot, Entry}

impl EntryRoot {
    pub fn empty() -> Self {
        Self {
            children: Vector::new(),
            selected: Vector::new(),
        }
    }

    pub fn new(path: impl AsRef<Path>) -> io::Result<Self> {
        let children = fs::read_dir(path)?
            .map(|entry| Entry::new(entry?))
            .collect::<io::Result<Vector<_>>>()?;
        Ok(Self {
            children,
            selected: Vector::new(),
        })
    }
}


#[derive(Debug)]
enum Msg {
    NewPath(PathBuf),
}

pub struct FileWatcher<T> {
    path: Option<PathBuf>,
    tree: WidgetPod<
        EntryRoot,
        Tree<
            EntryRoot,
            Entry,
            entry_derived_lenses::expanded,
            entry_root_derived_lenses::selected,
            1,
        >,
    >,
    root: EntryRoot,
    tx: Option<Sender<Msg>>,
    update_closure: Box<dyn Fn(&T) -> Option<PathBuf>>,
}

impl<T> FileWatcher<T> {
    pub fn new(update_closure: impl Fn(&T) -> Option<PathBuf> + 'static) -> Self {
        let tree = Tree::new(
            [Label::new("Hello3")],
            [Arc::new(|| {
                Label::dynamic(|data: &Entry, _env| data.name.clone()).boxed()
            })],
            Entry::expanded,
            EntryRoot::selected,
        )
        .set_sizes([400.]);
        Self {
            path: None,
            tree: WidgetPod::new(tree),
            tx: None,
            root: EntryRoot::empty(),
            update_closure: Box::new(update_closure),
        }
    }

    pub fn set_path(&mut self, path: Option<PathBuf>) {
        // We seend the path to the thread
        if self.path == path {
            return;
        }

        match &self.path {
            Some(path) => {
                if let Some(tx) = &self.tx {
                    tx.send(Msg::NewPath(path.clone())).unwrap()
                };
            }
            None => {
                // TODO reset
            }
        }
    }
}

impl<T: Data> Widget<T> for FileWatcher<T> {
    fn event(&mut self, ctx: &mut EventCtx, event: &Event, _: &mut T, env: &Env) {
        match event {
            Event::Command(command) if command.is(NEW_ROOT) => {
                ctx.set_handled();
                match command.get_unchecked(NEW_ROOT).take().unwrap() {
                    Ok(new_root) => {
                        self.root = new_root;
                        ctx.request_update();
                    }
                    Err(err) => {
                        dbg!(err);
                    }
                }
                return;
            }
            _ => (),
        }
        self.tree.event(ctx, event, &mut self.root, env)
    }

    fn lifecycle(&mut self, ctx: &mut LifeCycleCtx, event: &LifeCycle, data: &T, env: &Env) {
        if let LifeCycle::WidgetAdded = event {
            let (tx, rx) = crossbeam_channel::unbounded();
            let id = ctx.widget_id();
            let sink = ctx.get_external_handle();
            thread::spawn(move || {
                msg_thread(rx, id, sink);
            });
            self.tx = Some(tx);
            let new_path = (self.update_closure)(data);
            self.set_path(new_path);
        };
        self.tree.lifecycle(ctx, event, &self.root, env)
    }

    fn update(&mut self, ctx: &mut UpdateCtx, old_data: &T, data: &T, env: &Env) {
        self.tree.update(ctx, &self.root, env);
        if !old_data.same(data) {
            let new_path = (self.update_closure)(data);
            self.set_path(new_path);
        }
    }

    fn layout(&mut self, ctx: &mut LayoutCtx, bc: &BoxConstraints, _: &T, env: &Env) -> Size {
        let size = self.tree.layout(ctx, bc, &self.root, env);
        self.tree.set_origin(ctx, &self.root, env, Point::ORIGIN);
        size
    }

    fn paint(&mut self, ctx: &mut PaintCtx, _: &T, env: &Env) {
        self.tree.paint(ctx, &self.root, env)
    }
}

fn msg_thread(rx: Receiver<Msg>, widget_id: WidgetId, sink: ExtEventSink) {
    let (notify_tx, notify_rx) = crossbeam_channel::unbounded();
    let mut watcher = notify::recommended_watcher(notify_tx).unwrap();

    let mut sel = Select::new();
    let main_thread = sel.recv(&rx);
    let notify_thread = sel.recv(&notify_rx);

    loop {
        let oper = sel.select();
        match oper.index() {
            i if i == main_thread => {
                if let Ok(msg) = oper.recv(&rx) {
                    match msg {
                        Msg::NewPath(path) => {
                            sink.submit_command(
                                NEW_ROOT,
                                SingleUse::new(EntryRoot::new(&path)),
                                Target::Widget(widget_id),
                            )
                            .unwrap();
                            watcher
                                .watch(path.as_path(), RecursiveMode::Recursive)
                                .unwrap()
                        }
                    }
                } else {
                    break;
                }
            }
            i if i == notify_thread => {
                if let Ok(event) = oper.recv(&notify_rx) {
                    // TODO make things update
                }
            }
            _ => unreachable!(),
        }
    }
}
