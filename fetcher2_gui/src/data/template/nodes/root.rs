use druid::im::Vector;
use druid::{Data, Lens};

use fetcher2::template::nodes::root::RootNode;

use crate::communication::Communication;
use crate::data::template::nodes::node::NodeData;
use crate::widgets::tree::root::{impl_simple_tree_root, TreeNodeRoot};
use crate::widgets::tree::DataNodeIndex;

#[derive(Data, Clone, Debug, Lens)]
pub struct RootNodeData {
    pub children: Vector<NodeData>,
    pub selected: Vector<DataNodeIndex>,
}

impl_simple_tree_root! {RootNodeData, NodeData}

impl RootNodeData {
    pub fn new(root: RootNode<Communication>) -> Self {
        let children: Vector<_> = root.children.into_iter().map(NodeData::new).collect();

        Self {
            children,
            selected: Vector::new(),
        }
    }
    pub fn empty() -> Self {
        Self {
            children: Vector::new(),
            selected: Vector::new(),
        }
    }
}
