use crate::error::Error;
use crate::expr::Expr;
use crate::network::{Network, NodeId, ParamId};
use specs::world::Builder;
use specs::{Component, VecStorage};
use crate::eval::EvalContext;
use std::rc::Rc;
use std::any::Any;


pub struct Param<T: ?Sized> {
    v: T
}

impl<T> Param<T> {
    //pub fn from_closure(f: impl Fn(&mut EvalContext));
}

/*
impl Param {
    pub fn new(expr: Expr) -> Param {
        Param { expr }
    }

    pub fn expr(&self) -> &Expr {
        &self.expr
    }

    pub fn set_expr(&mut self, expr: Expr) {
        self.expr = expr;
    }
}*/

// trait Param
// trait TypedParam<Type=T>: Param
// OR
// Param<Value>
// Param<f32>
// Param<Any>
//
// NodeOps impls Param<Any>, downcastable to Param<<concrete type>>
// Can coerce Param<T> where T: Any to Param<dyn Any> (unsizing?)

// trait Param
// trait TypedParam<T>: Param
// downcast Param -> TypedParam<T>

// SimpleParam
// -> impl TypedParam
// -> impl Param

/// A set of parameters to update.
pub struct ParamUpdateSet
{
    dirty: Vec<ParamId>,
    /// Ok(true) if successfully updated, Ok(false) if not necessary to update, Err(_)
    /// if error during update.
    results: Vec<Result<bool,Error>>
}

//--------------------------------------------------------------------------------------------------
pub trait NodeOps {
    /// Returns the name of the node.
    fn name(&self) -> &str;
    /// Returns the display name of the node.
    fn display_node(&self) -> &str {
        self.name()
    }
    /// Signals to the node that one or more dependencies of this node have changed.
    ///
    /// The ParamUpdateSet contains the set of parameters (identified by nodeID+paramID) that were modified.
    /// The node should update its own parameters and signal which ones have changed through the
    /// provided ParamUpdateSet.
    fn dependencies_updated(&mut self, updates: &mut ParamUpdateSet);

    // issue here: node ops mut borrows storage of nodes, but needs access to params of other nodes
    // -> could extract (move out) node from storage before, and put it back afterwards
    //      -> storage not borrowed anymore
    //      -> feels like cheating, but why not
    //      -> should be relatively easy and fast to extract and put back
    //      -> invalidates pointers (but unlikely to have any pointers anyway: borrowck would prevent such designs)
    // -> other solution: don't borrow mutably node storage, go purely functional
    //      -> update_param sees updated values, returns new values of parameters or error
    //      -> new values of params updated later, all at once (lazy update?)
    //      -> collect updates in a (nodeID,paramID) -> value map
    // -> will happen once per frame, so make it efficient and fast
    //      -> how many param calculations per frame (or per second)
    //      -> say 1000000 params to recalc per frame?
    //
    // dependencies_updated should be called with a context
    // - the context is the full network + whatever custom contexts (identified by type) are relevant
    //      - can query contexts by type (but no typeid for non-'static structs...)
    //      - custom contexts must be able to contain refs to elements that only live for the duration of the traversal.
    //      -> this is not possible
    // - OR:
    //      - implement node logic in specs::System instead of generic callbacks
    //      - one system (+storage/component) for every node type
    //          - what about plugin nodes?
    //          - can register components at runtime
    //      - issue: on param update, must call all systems
    //          - costly? can't do it all at once because of dependencies between nodes (of different types)

    /*/// Returns a reference to a named parameter of the node.
    fn param(&self, name: &str) -> Result<&Param<dyn Any>, Error>;
    /// Returns a mutable reference to a named paramter of the node.
    fn param_mut(&mut self, name: &str) -> Result<&mut Param<dyn Any>, Error>;*/
}

impl<'a> Any<'a> {
    fn downcast_ref<T: 'a>(&self) -> Option<&T> { ... }

    // problem: T: 'a is always OK with 'static
    // want some 'b such that 'a outlives 'b, but the only constraint we can add on the type is T:'b, which is T outlives 'b, which is OK for static
    // (ran into similar issue with the plugin system)
    //
    // T must not be a type, but a type constructor => 'a -> T<'a>; but no higher-kinded type params in rust
    //
    // But can have T: BoundedAny<'a>
}
