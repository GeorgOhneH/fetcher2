use druid::{Data, Lens};
use druid::im::Vector;
use fetcher2::template::nodes::root::{RawRootNode, RootNode};

use crate::communication::Communication;
use crate::data::template_edit::nodes::node::NodeEditData;
use crate::edit_window::NodePosition;
use crate::widgets::tree::DataNodeIndex;
use crate::widgets::tree::root::{impl_simple_tree_root, TreeNodeRoot};

#[derive(Data, Clone, Debug, Lens)]
pub struct RootNodeEditData {
    pub children: Vector<NodeEditData>,
    pub selected: Vector<DataNodeIndex>,
}

impl_simple_tree_root! {RootNodeEditData, NodeEditData}

impl RootNodeEditData {
    pub fn new(root: RootNode<Communication>) -> Self {
        let children: Vector<_> = root.children.into_iter().map(NodeEditData::new).collect();

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

    pub fn remove(&mut self, idx: &[usize]) -> NodeEditData {
        match idx.len() {
            0 => panic!("Can't remove the root node"),
            1 => self.children.remove(idx[0]),
            _ => self.children[idx[0]].remove(&idx[1..]),
        }
    }

    pub fn insert_node(&mut self, idx: &[usize], pos: NodePosition) {
        match pos {
            NodePosition::Child => self.insert_child(idx),
            NodePosition::Above => self.insert_sibling(idx, 0),
            NodePosition::Below => self.insert_sibling(idx, 1),
        }
    }

    pub fn insert_sibling(&mut self, idx: &[usize], offset: usize) {
        match idx.len() {
            0 => panic!("Can't do this"),
            1 => self
                .children
                .insert(idx[0] + offset, NodeEditData::empty(true)),
            _ => self.children[idx[0]].insert_sibling(&idx[1..], offset),
        }
    }

    pub fn insert_child(&mut self, idx: &[usize]) {
        match idx.len() {
            0 => self.children.push_back(NodeEditData::empty(true)),
            _ => self.children[idx[0]].insert_child(&idx[1..]),
        }
    }
}
