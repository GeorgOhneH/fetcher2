use std::path::PathBuf;
use std::sync::Arc;

use druid::widget::Controller;
use druid::SingleUse;
use druid::{Env, Event, EventCtx, LifeCycle, LifeCycleCtx, Widget};

use crate::background_thread::{NEW_TEMPLATE, NODE_EVENT};
use crate::controller::{Msg, MSG_THREAD};
use crate::data::template::nodes::root::RootNodeData;
use crate::data::AppData;

pub struct TemplateController {}

impl TemplateController {
    pub fn new() -> Self {
        Self {}
    }

    fn new_root(data: &mut AppData, root: RootNodeData, path: Option<PathBuf>) {
        let arc_path = path.map(Arc::new);
        if let Some(new_path) = &arc_path {
            data.recent_templates.retain(|path| path != new_path);
            data.recent_templates.push_front(new_path.clone());
        }
        data.template.root = root;
        data.template.save_path = arc_path;
    }
}

impl<W: Widget<AppData>> Controller<AppData, W> for TemplateController {
    fn event(
        &mut self,
        child: &mut W,
        ctx: &mut EventCtx,
        event: &Event,
        data: &mut AppData,
        env: &Env,
    ) {
        match event {
            Event::Command(cmd) if cmd.is(NODE_EVENT) => {
                ctx.set_handled();
                let node_event = cmd.get_unchecked(NODE_EVENT).take().unwrap();
                data.template
                    .node_mut(&node_event.idx, |node, _| node.update_node(node_event.kind));
                return;
            }
            Event::Command(cmd) if cmd.is(NEW_TEMPLATE) => {
                ctx.set_handled();
                let (template_root, path) = cmd.get_unchecked(NEW_TEMPLATE).take().unwrap();
                Self::new_root(data, template_root, path);
                ctx.request_update();
                return;
            }
            _ => (),
        }
        child.event(ctx, event, data, env)
    }

    fn lifecycle(
        &mut self,
        child: &mut W,
        ctx: &mut LifeCycleCtx,
        event: &LifeCycle,
        data: &AppData,
        env: &Env,
    ) {
        if let LifeCycle::WidgetAdded = event {
            if let Some(last) = data.recent_templates.iter().next() {
                ctx.submit_command(MSG_THREAD.with(SingleUse::new(Msg::NewTemplateByPath(
                    (*last.clone()).clone(),
                ))))
            }
        }
        child.lifecycle(ctx, event, data, env)
    }
}
