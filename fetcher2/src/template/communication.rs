use crate::template::nodes::node::{NodeEvent, NodeEventKind};
use crate::template::NodeIndex;
use tokio::sync::mpsc::Sender;

pub trait RawCommunicationExt<T: CommunicationExt>: Clone {
    fn with_idx(self, idx: NodeIndex) -> T;
}

pub trait CommunicationExt: Clone + Send + Sync + 'static {
    fn send_event<T: Into<NodeEventKind>>(&self, event: T);
}

#[derive(Debug, Clone)]
pub struct RootNotifier {
    idx: NodeIndex,
    tx: Sender<NodeEvent>,
}

impl RootNotifier {
    pub fn new(tx: Sender<NodeEvent>, idx: NodeIndex) -> Self {
        Self { idx, tx }
    }
    pub async fn notify(&self, event: impl Into<NodeEventKind>) {
        self.tx
            .send(NodeEvent::new(event.into(), self.idx.clone()))
            .await
            .expect("Receiver was closed")
    }
}
