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

use crate::template::node_type::site::TaskMsg;
use crate::template::nodes::node::NodeEvent;
use crate::template::{NodeIndex, Template};
use crate::{Result, TError};

pub trait RawCommunicationExt<T: CommunicationExt>: Clone {
    fn with_idx(self, idx: NodeIndex) -> T;
}

pub trait CommunicationExt: Clone + Send + Sync + 'static {
    fn send_event<T: Into<NodeEvent>>(&self, event: T);
}
