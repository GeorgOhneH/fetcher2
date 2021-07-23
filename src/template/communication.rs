use std::convert::TryFrom;
use std::fmt::{Debug, Display, Formatter};
use std::sync::Arc;

use druid::kurbo::{BezPath, Size};
use druid::piet::{LineCap, LineJoin, RenderContext, StrokeStyle};
use druid::widget::Label;
use druid::{theme, ExtEventSink, Selector, SingleUse, WidgetId};
use druid::{
    BoxConstraints, Data, Env, Event, EventCtx, LayoutCtx, LifeCycle, LifeCycleCtx, PaintCtx,
    Point, UpdateCtx, Widget, WidgetPod,
};

use crate::template::node_type::site::{Msg, SiteEvent};
use crate::template::nodes::node_widget::{NodeWidget};
use crate::template::nodes::root_widget::{RootNodeData, RootNodeWidget};
use crate::template::Template;
use crate::{Result, TError};
use druid_widget_nursery::{selectors, Wedge};
use std::path::{Path, PathBuf};
use crate::template::nodes::node::NodeEvent;

pub const NODE_EVENT: Selector<SingleUse<NodeEvent>> = Selector::new("fetcher2.communucation.node_event");

#[derive(Clone)]
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

    pub fn send_event<T: Into<NodeEvent>>(&self, event: T) -> Result<()> {
        self.sink
            .as_ref()
            .unwrap()
            .submit_command(NODE_EVENT, SingleUse::new(event.into()), self.id.unwrap())?;
        Ok(())
    }
}

impl Debug for WidgetCommunication {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str(&format!(
            "WidgetCommunication {{ WidgetId {:?}, some_sink: {:?} }}",
            self.id,
            self.sink.is_some()
        ))
    }
}
