use std::collections::HashSet;
use std::future::Future;
use std::mem;
use std::path::PathBuf;
use std::sync::Arc;

use druid::{ExtEventSink, Selector, SingleUse, Target};
use druid_widget_nursery::selectors;
use futures::future::BoxFuture;
use futures::future::{AbortHandle, Abortable, Aborted};
use futures::prelude::stream::FuturesUnordered;
use futures::{FutureExt, StreamExt};
use tokio::sync::mpsc::Receiver;
use tokio::task::JoinHandle;

use fetcher2::settings::DownloadSettings;
use fetcher2::template::nodes::node::NodeEvent;
use fetcher2::template::{Prepared, Template, UnPrepared};
use fetcher2::TError;

use crate::controller::Msg;
use crate::data::template::nodes::root::RootNodeData;
use crate::data::template_edit::nodes::root::RootNodeEditData;
use crate::widgets::tree::NodeIndex;

selectors! {
    NEW_TEMPLATE: SingleUse<(RootNodeData, Option<PathBuf>)>,
    NEW_EDIT_TEMPLATE: SingleUse<(RootNodeEditData, Option<PathBuf>)>,

    MSG_FROM_THREAD: SingleUse<ThreadMsg>,
}

// TODO: use tokens for templates to make sure it will work correctly
pub const NODE_EVENT: Selector<SingleUse<NodeEvent>> =
    Selector::new("fetcher2.communucation.node_event");

pub enum ThreadMsg {
    SettingsRequired,
    TemplateLoadingError(TError),
    TemplateSaveError(TError),
}

enum RunType {
    Root,
    Indexes(HashSet<NodeIndex>),
}

enum PostCommand {
    None,
    RunPrepare,
}

enum TemplateState {
    Prepared(Template<Prepared>),
    UnPrepared(Template<UnPrepared>),
}

impl TemplateState {
    pub fn is_prepared(&self) -> bool {
        matches!(self, Self::Prepared(_))
    }

    pub async fn inform_of_cancel(&self) {
        match self {
            Self::Prepared(template) => template.inform_of_cancel().await,
            Self::UnPrepared(template) => template.inform_of_cancel().await,
        }
    }

    pub async fn save(&self) -> Result<(), TError> {
        match self {
            Self::Prepared(template) => template.save().await,
            Self::UnPrepared(template) => template.save().await,
        }
    }

    pub async fn prepare(&mut self, dsettings: Arc<DownloadSettings>) -> bool {
        if let TemplateState::UnPrepared(template) = self {
            match mem::take(template).prepare(dsettings.clone()).await {
                Ok(prepared_template) => {
                    *self = TemplateState::Prepared(prepared_template);
                    true
                }
                Err(unprepared_template) => {
                    *self = TemplateState::UnPrepared(unprepared_template);
                    false
                }
            }
        } else {
            true
        }
    }
}

struct TemplateData {
    pub template_state: TemplateState,
    pub handle: JoinHandle<()>,
}

impl TemplateData {
    pub fn empty() -> Self {
        Self {
            template_state: TemplateState::UnPrepared(Template::empty()),
            handle: tokio::spawn(async {}),
        }
    }

    pub async fn inform_of_cancel(&self) {
        self.template_state.inform_of_cancel().await
    }

    pub async fn save(&self) -> Result<(), TError> {
        self.template_state.save().await
    }

    pub fn new(
        template: Template<UnPrepared>,
        rx: Receiver<NodeEvent>,
        sink: ExtEventSink,
    ) -> Self {
        Self {
            template_state: TemplateState::UnPrepared(template),
            handle: tokio::spawn(forward_msgs(rx, sink)),
        }
    }

    pub async fn replace(
        &mut self,
        template: Template<UnPrepared>,
        tx: Receiver<NodeEvent>,
        sink: ExtEventSink,
    ) {
        self.template_state = TemplateState::UnPrepared(template);

        let dummy_handle = tokio::spawn(async {});
        let old_handle = mem::replace(&mut self.handle, dummy_handle);
        // TODO not unwrap
        old_handle.await.unwrap();
        let new_handle = tokio::spawn(forward_msgs(tx, sink));
        self.handle = new_handle;
    }
}

pub fn background_main(rx: flume::Receiver<Msg>, r: crossbeam_channel::Receiver<ExtEventSink>) {
    let sink = r.recv().expect("Should always work");
    tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .unwrap()
        .block_on(manager(rx, sink));
}

async fn manager(gui_rx: flume::Receiver<Msg>, sink: ExtEventSink) {
    let template_data = TemplateData::empty();
    let template_data = tokio::sync::RwLock::new(template_data);
    let mut dsettings: Option<Arc<DownloadSettings>> = None;

    let mut futs = FuturesUnordered::new();
    let mut abort_handles = Vec::new();

    loop {
        tokio::select! {
            Ok(msg) = gui_rx.recv_async() => {
                dbg!("{:?}", &msg);
                match msg {
                    Msg::StartAll => {
                        with_settings(
                            |settings| run_template(&template_data, settings, RunType::Root),
                            dsettings.clone(),
                            &mut futs,
                            &mut abort_handles,
                            sink.clone()
                        );
                    },
                    Msg::StartByIndex(indexes) => {
                        with_settings(
                            |settings| run_template(&template_data, settings, RunType::Indexes(indexes)),
                            dsettings.clone(),
                            &mut futs,
                            &mut abort_handles,
                            sink.clone()
                        );
                    },
                    Msg::Cancel => {
                        cancel_all(&mut abort_handles);
                        let fut = async { template_data.read().await.template_state.inform_of_cancel().await; PostCommand::None };
                        add_new_future(fut, &mut futs, &mut abort_handles);
                    },
                    Msg::NewSettings(new_settings) => {
                        dsettings = Some(Arc::new(new_settings));
                        with_settings(
                            |settings| prepare_template(&template_data, settings),
                            dsettings.clone(),
                            &mut futs,
                            &mut abort_handles,
                            sink.clone()
                        );
                    },
                    Msg::NewTemplate((new_template, new_rx)) => {
                        cancel_all(&mut abort_handles);
                        let fut = replace_template(&template_data, new_template, new_rx, sink.clone());
                        add_new_future(fut, &mut futs, &mut abort_handles);
                    },
                    Msg::NewTemplateByPath(path) => {
                        cancel_all(&mut abort_handles);
                        let fut = replace_template_by_path(&template_data, path, sink.clone());
                        add_new_future(fut, &mut futs, &mut abort_handles);
                    },
                    Msg::ExitAndSave => {
                        cancel_all(&mut abort_handles);
                        break;
                    }
                }
            }
            Some(result) = futs.next() => {
                if let Ok(cmd) = result {
                    handle_post_cmd(
                        cmd,
                        &template_data,
                        dsettings.clone(),
                        &mut futs,
                        &mut abort_handles,
                        sink.clone(),
                    )
                }
            },
            else => {
                dbg!("BREAK LOOP");
                break
            },
        }
    }

    while futs.next().await.is_some() {}
    // We use write so we are sure the other operations are finished
    template_data.write().await.save().await.unwrap();

    println!("GRACEFUL EXIT");
}

async fn forward_msgs(mut rx: Receiver<NodeEvent>, sink: ExtEventSink) {
    while let Some(event) = rx.recv().await {
        dbg!(&event);

        sink.submit_command(NODE_EVENT, SingleUse::new(event), Target::Global)
            .expect("Main Thread existed before this one");
    }
}

fn handle_post_cmd<'a>(
    cmd: PostCommand,
    template_data: &'a tokio::sync::RwLock<TemplateData>,
    dsettings: Option<Arc<DownloadSettings>>,
    mut futs: &mut FuturesUnordered<BoxFuture<'a, std::result::Result<PostCommand, Aborted>>>,
    mut abort_handles: &mut Vec<AbortHandle>,
    sink: ExtEventSink,
) {
    match cmd {
        PostCommand::None => (),
        PostCommand::RunPrepare => {
            with_settings(
                |settings| prepare_template(template_data, settings),
                dsettings,
                &mut futs,
                &mut abort_handles,
                sink,
            );
        }
    }
}

fn with_settings<'a, T: Future<Output = PostCommand> + Send + 'a>(
    fut: impl FnOnce(Arc<DownloadSettings>) -> T,
    dsettings: Option<Arc<DownloadSettings>>,
    futs: &mut FuturesUnordered<BoxFuture<'a, std::result::Result<PostCommand, Aborted>>>,
    abort_handles: &mut Vec<AbortHandle>,
    sink: ExtEventSink,
) {
    if let Some(dsettings) = dsettings {
        add_new_future(fut(dsettings), futs, abort_handles);
    } else {
        sink.submit_command(
            MSG_FROM_THREAD,
            SingleUse::new(ThreadMsg::SettingsRequired),
            Target::Global,
        )
        .unwrap()
    }
}

fn add_new_future<'a>(
    future: impl Future<Output = PostCommand> + Send + 'a,
    futs: &mut FuturesUnordered<BoxFuture<'a, std::result::Result<PostCommand, Aborted>>>,
    abort_handles: &mut Vec<AbortHandle>,
) {
    let (abort_handle, abort_registration) = AbortHandle::new_pair();
    let future = Abortable::new(future, abort_registration).boxed();
    abort_handles.push(abort_handle);
    futs.push(future);
}

fn cancel_all(abort_handles: &mut Vec<AbortHandle>) {
    for handle in abort_handles.iter() {
        handle.abort();
    }
    abort_handles.clear();
}

async fn replace_template_by_path(
    old_template_data: &tokio::sync::RwLock<TemplateData>,
    path: PathBuf,
    sink: ExtEventSink,
) -> PostCommand {
    dbg!("starting load");
    match Template::load(path.as_path()).await {
        Ok((new_template, new_rx)) => {
            replace_template(old_template_data, new_template, new_rx, sink).await
        }
        Err(err) => {
            sink.submit_command(
                MSG_FROM_THREAD,
                SingleUse::new(ThreadMsg::TemplateLoadingError(err)),
                Target::Global,
            )
            .unwrap();
            PostCommand::None
        }
    }
}

async fn replace_template(
    old_template_data: &tokio::sync::RwLock<TemplateData>,
    new_template: Template<UnPrepared>,
    new_rx: Receiver<NodeEvent>,
    sink: ExtEventSink,
) -> PostCommand {
    dbg!("replace load");
    old_template_data.read().await.inform_of_cancel().await;
    let mut wl = old_template_data.write().await;
    sink.submit_command(
        NEW_TEMPLATE,
        SingleUse::new((
            RootNodeData::new(&new_template.root),
            new_template.save_path.clone(),
        )),
        Target::Global,
    )
    .unwrap();
    sink.submit_command(
        NEW_EDIT_TEMPLATE,
        SingleUse::new((
            RootNodeEditData::new(&new_template.root),
            new_template.save_path.clone(),
        )),
        Target::Global,
    )
    .unwrap();
    if let Err(err) = wl.save().await {
        sink.submit_command(
            MSG_FROM_THREAD,
            SingleUse::new(ThreadMsg::TemplateSaveError(err)),
            Target::Global,
        )
        .unwrap()
    }
    if let Err(err) = new_template.save().await {
        sink.submit_command(
            MSG_FROM_THREAD,
            SingleUse::new(ThreadMsg::TemplateSaveError(err)),
            Target::Global,
        )
        .unwrap()
    }
    wl.replace(new_template, new_rx, sink).await;
    dbg!("finished");
    PostCommand::RunPrepare
}

async fn run_template(
    template_data: &tokio::sync::RwLock<TemplateData>,
    dsettings: Arc<DownloadSettings>,
    ty: RunType,
) -> PostCommand {
    loop {
        let rl = template_data.read().await;
        if let TemplateState::Prepared(template) = &rl.template_state {
            match ty {
                RunType::Root => template.run_root(dsettings.clone()).await,
                RunType::Indexes(ref indexes) => template.run(dsettings.clone(), indexes).await,
            };
            return PostCommand::None;
        }
        drop(rl);
        if !template_data
            .write()
            .await
            .template_state
            .prepare(dsettings.clone())
            .await
        {
            return PostCommand::None;
        }
    }
}
async fn prepare_template(
    template_data: &tokio::sync::RwLock<TemplateData>,
    dsettings: Arc<DownloadSettings>,
) -> PostCommand {
    dbg!("start prepare");
    template_data
        .write()
        .await
        .template_state
        .prepare(dsettings.clone())
        .await;
    PostCommand::None
}
