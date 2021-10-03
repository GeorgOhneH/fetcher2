use std::convert::TryFrom;
use std::fmt::{Debug, Display, Formatter};
use std::path::{Path, PathBuf};
use std::sync::Arc;

use druid::kurbo::{BezPath, Size};
use druid::piet::{LineCap, LineJoin, RenderContext, StrokeStyle};
use druid::widget::Label;
use druid::{theme, ExtEventSink, Selector, SingleUse, Target, WidgetId};
use druid::{
    BoxConstraints, Data, Env, Event, EventCtx, LayoutCtx, LifeCycle, LifeCycleCtx, PaintCtx,
    Point, UpdateCtx, Widget, WidgetPod,
};
use druid_widget_nursery::{selectors, Wedge};

use crate::template::nodes::root_data::RootNodeData;
use crate::widgets::tree::NodeIndex;
use crate::{Result, TError};
use fetcher2::template::communication::{CommunicationExt, RawCommunicationExt};
use fetcher2::template::nodes::node::NodeEvent;

// TODO: use tokens for templates to make sure it will work correctly
pub const NODE_EVENT: Selector<SingleUse<(NodeEvent, NodeIndex)>> =
    Selector::new("fetcher2.communucation.node_event");

#[derive(Clone)]
pub struct RawCommunication {
    sink: ExtEventSink,
}

impl RawCommunication {
    pub fn new(sink: ExtEventSink) -> Self {
        Self { sink }
    }
}

impl RawCommunicationExt<Communication> for RawCommunication {
    fn with_idx(self, idx: NodeIndex) -> Communication {
        Communication::new(self.sink, idx)
    }
}

impl Debug for RawCommunication {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str("RawCommunication")
    }
}

#[derive(Clone)]
pub struct Communication {
    sink: ExtEventSink,
    idx: NodeIndex,
}

impl Communication {
    pub fn new(sink: ExtEventSink, idx: NodeIndex) -> Self {
        Self { sink, idx }
    }
}
impl CommunicationExt for Communication {
    fn send_event<T: Into<NodeEvent>>(&self, event: T) {
        self.sink
            .submit_command(
                NODE_EVENT,
                SingleUse::new((event.into(), self.idx.clone())),
                Target::Global,
            )
            .expect("Main Thread existed before this one");
    }
}

impl Debug for Communication {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str("WidgetCommunication")
    }
}
