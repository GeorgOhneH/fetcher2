use druid::im::Vector;
use druid::{Data, Lens};
use fetcher2::template::nodes::root::RootNode;

use crate::data::template::nodes::node::NodeData;
use crate::widgets::tree::node::TreeNode;
use crate::widgets::tree::root::TreeNodeRoot;
use crate::widgets::tree::NodeIndex;

#[derive(Data, Clone, Debug, Lens, Default, TreeNodeRoot)]
pub struct RootNodeData {
    pub children: Vector<NodeData>,
    pub selected: Vector<NodeIndex>,
}

impl RootNodeData {
    pub fn new(root: &RootNode) -> Self {
        let children: Vector<_> = root.children.iter().map(NodeData::new).collect();

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
