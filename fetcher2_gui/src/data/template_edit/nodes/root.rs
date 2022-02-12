use druid::{Data, Lens};
use druid::im::Vector;
use fetcher2::template::nodes::root::{RawRootNode, RootNode};

use crate::data::template_edit::nodes::node::NodeEditData;
use crate::edit_window::NodePosition;
use crate::widgets::tree::NodeIndex;
use crate::widgets::tree::root::TreeNodeRoot;
use crate::widgets::tree::node::TreeNode;

#[derive(Data, Clone, Debug, Lens, Default, TreeNodeRoot)]
pub struct RootNodeEditData {
    pub children: Vector<NodeEditData>,
    pub selected: Vector<NodeIndex>,
}


impl RootNodeEditData {
    pub fn new(root: &RootNode) -> Self {
        let children: Vector<_> = root.children.iter().map(NodeEditData::new).collect();

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

    pub fn raw(self) -> RawRootNode {
        let children = self
            .children
            .into_iter()
            .filter_map(|child| child.raw())
            .collect();
        RawRootNode { children }
    }

    pub fn remove(&mut self, idx: &NodeIndex) -> NodeEditData {
        let slice = idx.iter().map(|i| *i).collect::<Vec<_>>();
        match slice.len() {
            0 => panic!("Can't remove the root node"),
            1 => self.children.remove(slice[0]),
            _ => self.children[slice[0]].remove(&slice[1..]),
        }
    }

    pub fn insert_node(&mut self, idx: &NodeIndex, pos: NodePosition) {
        match pos {
            NodePosition::Child => self.insert_child(idx),
            NodePosition::Above => self.insert_sibling(idx, 0),
            NodePosition::Below => self.insert_sibling(idx, 1),
        }
    }

    pub fn insert_sibling(&mut self, idx: &NodeIndex, offset: usize) {
        let slice = idx.iter().map(|i| *i).collect::<Vec<_>>();
        match slice.len() {
            0 => panic!("Can't do this"),
            1 => self
                .children
                .insert(slice[0] + offset, NodeEditData::empty(true)),
            _ => self.children[slice[0]].insert_sibling(&slice[1..], offset),
        }
    }

    pub fn insert_child(&mut self, idx: &NodeIndex) {
        let slice = idx.iter().map(|i| *i).collect::<Vec<_>>();
        match slice.len() {
            0 => self.children.push_back(NodeEditData::empty(true)),
            _ => self.children[slice[0]].insert_child(&slice[1..]),
        }
    }
}
