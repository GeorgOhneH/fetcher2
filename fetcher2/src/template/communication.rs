use crate::template::nodes::node::NodeEvent;
use crate::template::NodeIndex;

pub trait RawCommunicationExt<T: CommunicationExt>: Clone {
    fn with_idx(self, idx: NodeIndex) -> T;
}

pub trait CommunicationExt: Clone + Send + Sync + 'static {
    fn send_event<T: Into<NodeEvent>>(&self, event: T);
}
