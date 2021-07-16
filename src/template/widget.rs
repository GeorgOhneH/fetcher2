use std::convert::TryFrom;
use std::fmt::{Display, Debug, Formatter};
use std::sync::Arc;

use druid::kurbo::{BezPath, Size};
use druid::piet::{LineCap, LineJoin, RenderContext, StrokeStyle};
use druid::{theme, WidgetId, ExtEventSink, Selector, SingleUse};
use druid::widget::Label;
use druid::{
    BoxConstraints, Data, Env, Event, EventCtx, LayoutCtx, LifeCycle, LifeCycleCtx, PaintCtx,
    Point, UpdateCtx, Widget, WidgetPod,
};


use druid_widget_nursery::{selectors, Wedge};
use crate::template::node::widget::{NodeData, TreeNodeWidget, RootNodeWidget, RootNodeData};
use crate::template::Template;
use std::path::{PathBuf, Path};
use crate::Result;


pub const PATH_UPDATED: Selector<SingleUse<PathBuf>> = Selector::new("blabla.blabla");

#[derive(Debug, Clone, Data)]
pub struct TemplateData {
    pub root: RootNodeData,
}

pub struct WidgetCommunication {
    pub sink: Option<ExtEventSink>,
    pub id: Option<WidgetId>,
}

impl WidgetCommunication {
    pub fn new() -> Self {
        Self {
            sink: None,
            id: None,
        }
    }

    pub async fn send_new_path(&self, path: PathBuf) -> Result<()> {
        let sink_clone = self.sink.clone().unwrap();
        let id_clone = self.id.unwrap().clone();
        tokio::task::spawn_blocking(move ||sink_clone.submit_command(PATH_UPDATED, SingleUse::new(path), id_clone)).await??;
        Ok(())
    }
}


impl Debug for WidgetCommunication {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str(&format!("WidgetCommunication {{ WidgetId {:?}, is_sink: {:?} }}", self.id, self.sink.is_some()))
    }
}

/// A tree widget for a collection of items organized in a hierachical way.
pub struct TemplateWidget
{
    /// The root node of this tree
    root: RootNodeWidget,
}

impl TemplateWidget {
    /// Create a new Tree widget
    pub fn new(root: RootNodeWidget) -> Self {
        Self {
            root,
        }
    }
}

// Implement the Widget trait for Tree
impl Widget<TemplateData> for TemplateWidget {
    fn event(&mut self, ctx: &mut EventCtx, event: &Event, data: &mut TemplateData, env: &Env) {
        // eprintln!("{:?}", event);
        self.root.event(ctx, event, &mut data.root, env);
    }

    fn lifecycle(&mut self, ctx: &mut LifeCycleCtx, event: &LifeCycle, data: &TemplateData, env: &Env) {
        self.root.lifecycle(ctx, event, &data.root, env);
    }

    fn update(&mut self, ctx: &mut UpdateCtx, old_data: &TemplateData, data: &TemplateData, env: &Env) {
        self.root.update(ctx, &old_data.root, &data.root, env);
    }

    fn layout(&mut self, ctx: &mut LayoutCtx, bc: &BoxConstraints, data: &TemplateData, env: &Env) -> Size {
        bc.constrain(self.root.layout(ctx, bc, &data.root, env))
    }

    fn paint(&mut self, ctx: &mut PaintCtx, data: &TemplateData, env: &Env) {
        let background_color = env.get(theme::BACKGROUND_LIGHT);
        let clip_rect = ctx.size().to_rect();
        ctx.fill(clip_rect, &background_color);
        self.root.paint(ctx, &data.root, env);
    }
}