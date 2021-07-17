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
use crate::template::nodes::node_widget::{NodeData, NodeWidget};
use crate::template::Template;
use std::path::{PathBuf, Path};
use crate::Result;
use crate::template::nodes::root_widget::{RootNodeData, RootNodeWidget};


pub const PATH_UPDATED: Selector<SingleUse<PathBuf>> = Selector::new("blabla.blabla");

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