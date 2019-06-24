pub mod id_tree;
pub mod node;
use crate::error::Error;
use crate::error::Result;
use crate::network::node::NodeOps;
use specs::world::Builder;
use specs::{Component, VecStorage};
use std::path::Path;

/// Identifiers for Nodes
#[derive(Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
#[repr(transparent)]
pub struct NodeId(specs::world::Entity);

/// Identifiers for a parameter within a node
#[derive(Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
#[repr(transparent)]
pub struct ParamId(u32);

type Depgraph = petgraph::graphmap::DiGraphMap<(NodeId, ParamId), ()>;

pub struct Node {
    /// Name of the node.
    ///
    /// Can be empty, in which case the node cannot be accessed by path, but only via an identifier.
    name: String,
    /// Parent node, None if this is the root node
    parent: Option<NodeId>,
    /// Child nodes. If it's empty, then the node is a leaf node. Otherwise, it also represents a subnetwork.
    children: Vec<NodeId>,
}

impl Node {
    /// Creates a root node (no parent).
    pub(super) fn new_root() -> Node {
        Node {
            name: String::new(),
            parent: None,
            children: Vec::new(),
        }
    }

    pub(super) fn with_parent(name: impl Into<String>, parent: NodeId) -> Node {
        Node {
            name: name.into(),
            parent: Some(parent),
            children: Vec::new(),
        }
    }

    /// Returns whether the node contains a subnetwork.
    pub fn is_subnet(&self) -> bool {
        !self.children.is_empty()
    }

    /// Returns the parent node.
    pub fn parent(&self) -> Option<NodeId> {
        self.parent
    }

    /// Returns whether the node is a root node.
    pub fn is_root(&self) -> bool {
        self.parent.is_none()
    }
}

impl Component for Node {
    type Storage = VecStorage<Self>;
}

pub struct Network {
    /// specs::World managing the node components.
    w: specs::World,

    /// Identifier of the root node.
    root: NodeId,

    /// Dependency graph between node parameters.
    ///
    /// It tracks the dependent values of parameters in nodes. (i.e. which parameters should be recomputed
    /// as soon as the value of a parameter changes).
    ///
    /// The graph should be invalidated as soon as a node is removed.
    depgraph: Depgraph,
}

pub trait IntoNodeId {
    fn resolve(&self, n: &NetworkScope) -> NodeId;
}

impl IntoNodeId for NodeId {
    fn resolve(&self, _: &NetworkScope) -> NodeId {
        *self
    }
}

impl Network {
    /// Creates a new node network.
    pub fn new() -> Network {
        let mut w = specs::World::new();

        // register all components that make up a node
        // the hierarchy links (parent and children)
        w.register::<Node>();

        // create root node
        let root = NodeId(w.create_entity().with(Node::new_root()).build());

        Network {
            w,
            depgraph: Depgraph::new(),
            root,
        }
    }

    /// Returns the root node identifier of this network.
    pub fn root_node(&self) -> NodeId {
        self.root
    }

    pub fn open_root(&mut self) -> NetworkScope {
        let id = self.root;
        NetworkScope { network: self, id }
    }

    fn add_named_node(&mut self, name: impl Into<String>, parent: NodeId) -> Result<NodeId> {
        let nid = NodeId(
            self.w
                .create_entity()
                .with(Node::with_parent(name, parent))
                .build(),
        );
        let mut node_storage = self.w.write_storage::<Node>();
        node_storage
            .get_mut(parent.0)
            .ok_or(Error::IdNotFound)?
            .children
            .push(nid);
        Ok(nid)
    }
}

/// A view of a network scoped to a particular reference node.
pub struct NetworkScope<'a> {
    network: &'a mut Network,
    id: NodeId,
}

impl<'a> NetworkScope<'a> {
    pub fn add_named_node(&mut self, name: impl Into<String>) -> NodeId {
        self.network.add_named_node(name, self.id).unwrap()
    }
}

/// Proxy context for modifying edits.
///
/// Use the methods on this object to add/remove nodes and modify parameters.
///
/// This will track all dirty nodes. It is created by [Network::modify], which will update
/// all dirty nodes upon returning from the closure.
struct NetworkProxy<'a> {
    n: &'a mut Network,
}

impl<'a> NetworkProxy<'a> {
    pub fn node_mut(&mut self, id: NodeId) -> Result<NodeProxy<'a>> {
        unimplemented!()
    }

    pub fn delete_node(&mut self, id: NodeId) -> Result<()> {
        unimplemented!()
    }
}

/// Proxy context for modifying a node.
///
/// Tracks modification of parameters and reports them in the dependency graph.
pub struct NodeProxy<'a> {
    depgraph: &'a mut Depgraph,
    node: &'a dyn NodeOps,
}
