use druid::widget::Controller;
use druid::{commands, SingleUse};
use druid::{Env, Event, EventCtx, Widget};
use druid_widget_nursery::selectors;
use fetcher2::settings::DownloadSettings;
use fetcher2::template::nodes::node::NodeEvent;
use fetcher2::template::{Template, UnPrepared};
use std::collections::HashSet;
use std::fmt::Debug;
use std::fs;
use std::io::Write;
use std::path::PathBuf;
use std::time::Duration;
use tokio::sync::mpsc::Receiver;

use crate::background_thread::{ThreadMsg, MSG_FROM_THREAD};
use crate::data::win::WindowState;
use crate::data::AppData;
use crate::utils::show_err;
use crate::widgets::tree::NodeIndex;
use crate::WINDOW_STATE_DIR;
use crate::{Result, TError};

selectors! {
    MSG_THREAD: SingleUse<Msg>
}

#[derive(Debug)]
pub enum Msg {
    StartAll,
    StartByIndex(HashSet<NodeIndex>),
    Cancel,
    NewSettings(DownloadSettings),
    NewTemplate((Template<UnPrepared>, Receiver<NodeEvent>)),
    NewTemplateByPath(PathBuf),
    ExitAndSave,
}

pub struct MainController {
    tx: flume::Sender<Msg>,
    load_err: Option<TError>,
}

impl MainController {
    pub fn new(load_err: Option<TError>, tx: flume::Sender<Msg>) -> Self {
        Self { tx, load_err }
    }
}

impl MainController {
    fn handle_thread_msg(ctx: &mut EventCtx, data: &AppData, env: &Env, thread_msg: ThreadMsg) {
        match thread_msg {
            ThreadMsg::SettingsRequired => ctx.submit_command(commands::SHOW_PREFERENCES),
            ThreadMsg::TemplateLoadingError(err) => {
                show_err(ctx, data, env, err, "Could not load template")
            }
            ThreadMsg::TemplateSaveError(err) => {
                show_err(ctx, data, env, err, "Could not save template")
            }
        };
    }

    fn save_window_state(app_state: &AppData) -> Result<()> {
        let serialized = ron::to_string(app_state)?;

        fs::create_dir_all(WINDOW_STATE_DIR.as_path().parent().expect(""))?;

        let mut f = fs::OpenOptions::new()
            .write(true)
            .truncate(true)
            .create(true)
            .open(WINDOW_STATE_DIR.as_path())?;
        f.write_all(serialized.as_bytes())?;
        Ok(())
    }
}

impl<W: Widget<AppData>> Controller<AppData, W> for MainController {
    fn event(
        &mut self,
        child: &mut W,
        ctx: &mut EventCtx,
        event: &Event,
        data: &mut AppData,
        env: &Env,
    ) {
        match event {
            Event::Command(cmd) if cmd.is(MSG_THREAD) => {
                ctx.set_handled();
                let msg = cmd.get_unchecked(MSG_THREAD).take().expect("");
                self.tx.send(msg).unwrap();
                return;
            }
            Event::Command(cmd) if cmd.is(MSG_FROM_THREAD) => {
                ctx.set_handled();
                let thread_msg = cmd.get_unchecked(MSG_FROM_THREAD).take().expect("");
                Self::handle_thread_msg(ctx, data, env, thread_msg)
            }
            Event::Command(cmd) if cmd.is(commands::OPEN_FILE) => {
                ctx.set_handled();
                let file_info = cmd.get_unchecked(commands::OPEN_FILE);
                self.tx
                    .send(Msg::NewTemplateByPath(file_info.path.clone()))
                    .expect("");
                return;
            }
            Event::WindowConnected => {
                ctx.request_timer(Duration::from_millis(100));
            }
            Event::Timer(_) => {
                if let Some(err) = self.load_err.take() {
                    show_err(ctx, data, env, err, "Could not load window state");
                }
            }
            Event::WindowCloseRequested => {
                data.main_window = Some(WindowState::from_win(ctx.window()));
            }
            Event::WindowDisconnected => {
                self.tx.send(Msg::ExitAndSave).expect("");
                Self::save_window_state(data).expect("Could not save AppData")
            }
            _ => (),
        }
        child.event(ctx, event, data, env)
    }
}
