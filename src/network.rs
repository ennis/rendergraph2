pub mod id_tree;
pub mod node;
use crate::error::Error;
use crate::error::Result;
use crate::network::node::NodeOps;
use slotmap::{SecondaryMap, SlotMap};
use std::any::Any;
use std::marker::PhantomData;
use std::path::Path;

slotmap::new_key_type! {
    pub struct GenericNodeId;
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
#[repr(transparent)]
pub struct NodeId<T>(pub GenericNodeId, PhantomData<T>);

impl<T> From<NodeId<T>> for GenericNodeId {
    fn from(id: NodeId<T>) -> Self {
        id.0
    }
}

type Depgraph = petgraph::graphmap::DiGraphMap<GenericNodeId, ()>;

pub struct NodeHierarchy {
    /// Name of the node.
    /// Can be empty, in which case the node cannot be accessed by path, but only via an identifier.
    name: String,
    /// Parent node, None if this is the root node
    parent: Option<GenericNodeId>,
    /// Child nodes. If it's empty, then the node is a leaf node. Otherwise, it also represents a subnetwork.
    children: Vec<GenericNodeId>,
}

impl NodeHierarchy {
    /// Creates a root node (no parent).
    pub(super) fn new_root() -> NodeHierarchy {
        NodeHierarchy {
            name: String::new(),
            parent: None,
            children: Vec::new(),
        }
    }

    pub(super) fn with_parent(name: impl Into<String>, parent: GenericNodeId) -> NodeHierarchy {
        NodeHierarchy {
            name: name.into(),
            parent: Some(parent),
            children: Vec::new(),
        }
    }

    /// Returns whether the node contains a subnetwork.
    pub fn has_subnet(&self) -> bool {
        !self.children.is_empty()
    }

    /// Returns the parent node.
    pub fn parent(&self) -> Option<GenericNodeId> {
        self.parent
    }

    /// Returns whether the node is a root node.
    pub fn is_root(&self) -> bool {
        self.parent.is_none()
    }
}

pub struct Network {
    hierarchy: slotmap::SlotMap<GenericNodeId, NodeHierarchy>,
    data: slotmap::SecondaryMap<GenericNodeId, Box<dyn Any>>,
    root: GenericNodeId,
    depgraph: Depgraph,
}

impl Network {
    /// Creates a new node network.
    pub fn new() -> Network {
        let mut hierarchy = SlotMap::with_key();
        let root = hierarchy.insert(NodeHierarchy::new_root());
        let data = SecondaryMap::new();

        Network {
            hierarchy,
            data,
            depgraph: Depgraph::new(),
            root,
        }
    }

    /// Returns the root node identifier of this network.
    pub fn root_node(&mut self) -> Node {
        let id = self.root;
        Node { network: self, id }
    }

    /// Creates a new node in the graph with the specified parent.
    fn new_node(&mut self, parent: GenericNodeId, name: impl Into<String>) -> GenericNodeId {
        self.hierarchy
            .insert(NodeHierarchy::with_parent(name, parent))
    }

    fn set_data<T: Any>(&mut self, node_id: GenericNodeId, data: T) -> NodeId<T> {
        self.data.insert(node_id, Box::new(data));
        NodeId(node_id, PhantomData)
    }
}

pub struct Node<'a> {
    network: &'a mut Network,
    id: GenericNodeId,
}

impl<'a> Node<'a> {
    pub fn set_data<T: Any>(&mut self, data: T) -> NodeId<T> {
        self.network.set_data(self.id, data)
    }

    pub fn subnet(&mut self) -> Subnet {
        Subnet {
            network: self.network,
            parent_node_id: self.id,
        }
    }

    pub fn id(&self) -> GenericNodeId {
        self.id
    }
}

pub struct Subnet<'a> {
    network: &'a mut Network,
    parent_node_id: GenericNodeId,
}

impl<'a> Subnet<'a> {
    pub fn new_node<'b>(&'b mut self, name: impl Into<String>) -> Node<'b>
    where
        'a: 'b,
    {
        let id = self.network.new_node(self.parent_node_id, name);
        Node {
            network: self.network,
            id,
        }
    }
}
