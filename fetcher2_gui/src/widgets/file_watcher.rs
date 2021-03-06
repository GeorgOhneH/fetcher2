use std::cmp::Ordering;
use std::ffi::OsStr;
use std::fs::Metadata;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::thread;
use std::time::SystemTime;
use std::{fs, io};

use chrono::{DateTime, Local};
use crossbeam_channel::{Receiver, Select, Sender};
use druid::im::Vector;
use druid::widget::Label;
use druid::{
    BoxConstraints, Data, Env, Event, EventCtx, ExtEventSink, LayoutCtx, Lens, LifeCycle,
    LifeCycleCtx, PaintCtx, Point, SingleUse, Size, Target, UpdateCtx, Widget, WidgetExt, WidgetId,
    WidgetPod,
};
use druid_widget_nursery::selectors;
use notify::{RecursiveMode, Watcher};

use crate::widgets::tree::node::TreeNode;
use crate::widgets::tree::root::TreeNodeRoot;
use crate::widgets::tree::{NodeIndex, Tree};

selectors! {
    NEW_ROOT: SingleUse<EntryRoot>,
    UPDATE: SingleUse<Vec<(PathBuf, EntryUpdate)>>,
}

fn find_child_by_name(children: &Vector<Entry>, name: &str) -> Option<usize> {
    let all: Vec<_> = children
        .iter()
        .enumerate()
        .filter_map(
            |(i, child)| {
                if &child.name == name {
                    Some(i)
                } else {
                    None
                }
            },
        )
        .collect();
    match all.len() {
        0 => None,
        1 => Some(all[0]),
        _ => panic!("name should be unique"),
    }
}

#[derive(Data, Copy, Clone, Debug, Eq, PartialEq)]
pub enum Type {
    Folder,
    File,
}

impl PartialOrd for Type {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Type {
    fn cmp(&self, other: &Self) -> Ordering {
        if self == other {
            Ordering::Equal
        } else if let Self::Folder = self {
            Ordering::Less
        } else {
            Ordering::Greater
        }
    }
}

pub struct UpdateData {
    size: String,
    created: String,
}

impl UpdateData {
    pub fn new(metadata: &Metadata) -> Self {
        Self {
            size: format_size(metadata.len()),
            created: metadata
                .created()
                .map(format_time)
                .unwrap_or("".to_string()),
        }
    }
}

#[derive(Data, Clone, Debug, Lens, Eq, PartialEq, TreeNode)]
pub struct Entry {
    name: String,
    children: Vector<Entry>,
    ty: Type,
    size: String,
    created: String,

    path: Arc<PathBuf>,
    expanded: bool,
}

impl PartialOrd for Entry {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Entry {
    fn cmp(&self, other: &Self) -> Ordering {
        if self.ty.cmp(&other.ty) == Ordering::Equal {
            self.name.cmp(&other.name)
        } else {
            self.ty.cmp(&other.ty)
        }
    }
}

impl Entry {
    pub fn new(name: String, ty: Type, size: String, created: String, path: PathBuf) -> Self {
        Self {
            name,
            ty,
            size,
            created,
            path: Arc::new(path),
            expanded: false,
            children: Vector::new(),
        }
    }
    pub fn build(entry: &fs::DirEntry) -> io::Result<Self> {
        let meta_data = entry.metadata()?;
        let name = entry.file_name().to_string_lossy().to_string();
        if meta_data.is_file() {
            Ok(Self {
                size: format_size(meta_data.len()),
                created: meta_data
                    .created()
                    .map(format_time)
                    .unwrap_or("".to_string()),
                children: Vector::new(),
                path: Arc::new(entry.path()),
                expanded: false,
                name,
                ty: Type::File,
            })
        } else {
            let mut children = fs::read_dir(entry.path())?
                .map(|child_entry| Entry::build(&child_entry?))
                .collect::<io::Result<Vector<_>>>()?;
            children.sort();
            Ok(Self {
                size: format_size(meta_data.len()),
                created: meta_data
                    .created()
                    .map(format_time)
                    .unwrap_or("".to_string()),
                children,
                path: Arc::new(entry.path()),
                expanded: false,
                name,
                ty: Type::Folder,
            })
        }
    }

    fn update_data(&mut self, ty: Type, data: UpdateData) -> bool {
        if &self.ty == &ty && &self.size == &data.size && &self.created == &data.created {
            return false;
        }
        self.ty = ty;
        self.size = data.size;
        self.created = data.created;
        true
    }
}

#[derive(Data, Clone, Debug, Lens, TreeNodeRoot)]
pub struct EntryRoot {
    #[data(eq)]
    path: Option<PathBuf>,
    children: Vector<Entry>,
    selected: Vector<NodeIndex>,
}

impl EntryRoot {
    pub fn empty() -> Self {
        Self {
            children: Vector::new(),
            selected: Vector::new(),
            path: None,
        }
    }

    pub fn new(path: PathBuf) -> Self {
        Self::build(&path).unwrap_or(Self {
            children: Vector::new(),
            path: Some(path),
            selected: Vector::new(),
        })
    }

    pub fn build(path: &PathBuf) -> io::Result<Self> {
        let mut children = fs::read_dir(&path)?
            .map(|child_entry| Entry::build(&child_entry?))
            .collect::<io::Result<Vector<_>>>()?;
        children.sort();
        Ok(Self {
            children,
            path: Some(path.clone()),
            selected: Vector::new(),
        })
    }

    fn update(&mut self, update: EntryUpdate, update_path: &Path) -> bool {
        if let Some(path) = &self.path {
            // TODO not unwrap
            let r_path = update_path.strip_prefix(path).unwrap();
            let components: Vec<_> = r_path.iter().collect();
            if components.is_empty() {
                return false;
            }
            match update {
                EntryUpdate::CreateUpdate(ty, data) => {
                    update_children(&mut self.children, &components, ty, data, path)
                }
                EntryUpdate::Remove => remove_children(&mut self.children, &components),
            }
        } else {
            false
        }
    }
}

pub enum EntryUpdate {
    CreateUpdate(Type, UpdateData),
    Remove,
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
            3,
        >,
    >,
    root: EntryRoot,
    tx: Option<Sender<Msg>>,
    update_closure: Box<dyn Fn(&T) -> Option<PathBuf>>,
}

impl<T> FileWatcher<T> {
    pub fn new(update_closure: impl Fn(&T) -> Option<PathBuf> + 'static) -> Self {
        let tree = Tree::new(
            [
                Label::new("Name"),
                Label::new("Size"),
                Label::new("Date Created"),
            ],
            [
                Arc::new(|| Label::dynamic(|data: &Entry, _env| data.name.clone()).boxed()),
                Arc::new(|| Label::dynamic(|data: &Entry, _env| data.size.clone()).boxed()),
                Arc::new(|| Label::dynamic(|data: &Entry, _env| data.created.clone()).boxed()),
            ],
            Entry::expanded,
            EntryRoot::selected,
        )
        .sizes([300., 300., 300.])
        .on_activate(|_ctx, root, _env, idx| {
            let node = root.node(idx);
            open::that_in_background(&*node.path);
        });
        Self {
            path: None,
            tree: WidgetPod::new(tree),
            tx: None,
            root: EntryRoot::empty(),
            update_closure: Box::new(update_closure),
        }
    }

    pub fn get_header_sizes(&self) -> [f64; 3] {
        self.tree.widget().get_sizes()
    }

    pub fn set_header_size(&mut self, sizes: [f64; 3]) {
        self.tree.widget_mut().set_sizes(sizes)
    }

    pub fn set_path(&mut self, path: Option<PathBuf>) {
        if self.path == path {
            return;
        }
        self.path = path;

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
            Event::Command(cmd) if cmd.is(NEW_ROOT) => {
                ctx.set_handled();
                self.root = cmd.get_unchecked(NEW_ROOT).take().unwrap();
                ctx.request_update();
                return;
            }
            Event::Command(cmd) if cmd.is(UPDATE) => {
                ctx.set_handled();
                for (path, up) in cmd.get_unchecked(UPDATE).take().unwrap() {
                    if self.root.update(up, &path) {
                        ctx.request_update();
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
    let mut watcher = notify::RecommendedWatcher::new(notify_tx).unwrap();

    let timer = timer::Timer::new();
    let (timer_tx, timer_rx) = crossbeam_channel::unbounded();

    let _guard = timer.schedule_repeating(chrono::Duration::milliseconds(100), move || {
        timer_tx.send(()).unwrap();
    });

    let mut updates = Vec::new();

    let mut sel = Select::new();
    let main_thread = sel.recv(&rx);
    let notify_thread = sel.recv(&notify_rx);
    let timer_thread = sel.recv(&timer_rx);

    let mut current_watch_path: Option<PathBuf> = None;

    loop {
        let oper = sel.select();
        match oper.index() {
            i if i == main_thread => {
                if let Ok(msg) = oper.recv(&rx) {
                    match msg {
                        Msg::NewPath(path) => {
                            if let Some(watch_path) = current_watch_path.as_ref() {
                                watcher.unwatch(watch_path.as_path()).unwrap()
                            }
                            current_watch_path = Some(
                                get_parent_exit(&path)
                                    .expect("Path must always have a valid parent")
                                    .to_owned(),
                            );
                            sink.submit_command(
                                NEW_ROOT,
                                SingleUse::new(EntryRoot::new(path.clone())),
                                Target::Widget(widget_id),
                            )
                            .unwrap();
                            watcher
                                .watch(
                                    current_watch_path.as_ref().unwrap().as_path(),
                                    RecursiveMode::Recursive,
                                )
                                .unwrap()
                        }
                    }
                } else {
                    break;
                }
            }
            i if i == notify_thread => match oper.recv(&notify_rx).unwrap() {
                Ok(event) => {
                    for path in event.paths {
                        let update = if let Ok(metadata) = path.metadata() {
                            let ty = match metadata.is_file() {
                                true => Type::File,
                                false => Type::Folder,
                            };
                            let data = UpdateData::new(&metadata);
                            (path, EntryUpdate::CreateUpdate(ty, data))
                        } else {
                            (path, EntryUpdate::Remove)
                        };
                        updates.push(update)
                    }
                }
                Err(err) => {
                    dbg!(err);
                }
            },
            i if i == timer_thread => {
                oper.recv(&timer_rx).unwrap();
                if !updates.is_empty() {
                    sink.submit_command(UPDATE, SingleUse::new(updates), Target::Widget(widget_id))
                        .unwrap();
                    updates = Vec::new();
                }
            }
            _ => unreachable!(),
        }
    }
}

fn get_parent_exit(path: &Path) -> Option<&Path> {
    if path.exists() {
        return Some(path);
    } else {
        path.parent().map(get_parent_exit).flatten()
    }
}

fn format_size(size: u64) -> String {
    bytesize::to_string(size, true)
}

fn format_time(time: SystemTime) -> String {
    let datetime: DateTime<Local> = time.into();
    datetime.format("%d.%m.%Y %T").to_string()
}

fn update_children(
    children: &mut Vector<Entry>,
    components: &[&OsStr],
    ty: Type,
    data: UpdateData,
    path: &PathBuf,
) -> bool {
    let name = components[0].to_string_lossy().to_string();
    let child_idx = find_child_by_name(children, name.as_str());
    match (components.len(), child_idx) {
        (0, _) => unreachable!(),
        (1, Some(child_idx)) => children.get_mut(child_idx).unwrap().update_data(ty, data),
        (1, None) => {
            let path = path.as_path().join(components[0]).to_path_buf();
            children.insert_ord(Entry::new(name, ty, data.size, data.created, path));
            true
        }
        (_, Some(child_idx)) => {
            let child = children.get_mut(child_idx).unwrap();

            update_children(
                &mut child.children,
                &components[1..],
                ty,
                data,
                &mut child.path,
            )
        }
        (_, None) => {
            let path = path.as_path().join(components[0]).to_path_buf();
            let mut child = Entry::new(
                name,
                Type::Folder,
                "".to_string(),
                format_time(SystemTime::now()),
                path,
            );
            update_children(
                &mut child.children,
                &components[1..],
                ty,
                data,
                &mut child.path,
            );
            children.insert_ord(child);
            true
        }
    }
}

fn remove_children(children: &mut Vector<Entry>, components: &[&OsStr]) -> bool {
    let name = components[0].to_string_lossy();
    if let Some(to_remove) = find_child_by_name(children, name.as_ref()) {
        match components.len() {
            0 => unreachable!(),
            1 => {
                children.remove(to_remove);
                true
            }
            _ => {
                if let Some(child) = children.get_mut(to_remove) {
                    remove_children(&mut child.children, &components[1..])
                } else {
                    false
                }
            }
        }
    } else {
        false
    }
}
