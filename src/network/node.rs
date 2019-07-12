use crate::error::Error;
use crate::eval::EvalContext;
use std::any::Any;
use std::rc::Rc;

//--------------------------------------------------------------------------------------------------
pub trait NodeOps {
    /// Returns the name of the node.
    fn name(&self) -> &str;
    /// Returns the display name of the node.
    fn display_node(&self) -> &str {
        self.name()
    }

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
    //0
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

    // Network types should have control over what is done (order of execution, etc)
    // Dependencies are interpreted by all systems as they see fit
    // Having only the NodeOps trait for interpreting a node graph is not very flexible (must cram everything into the context)
    //      - Whereas plug-in systems can build their own context and pass them to the nodes as they see fit
    //
    // On parameter change
    // - maybe parameters don't need a custom context?
    // - all data is going to be 'ectx scoped anyway during parameter change
    // - need custom context for execution, but maybe not for editor changes?
    // - in this case, just have a NodeParams component for params and a custom component for the network type

    //
    // IDEA: parameters are nodes within a subnet, exposed to the user as knobs
    // -> other nodes can depend on them without needing an eval context for the parent node
    // -> parameters are ValueNodes
    //
    // On creating a node, all params are also created in a subnet

    /*/// Returns a reference to a named parameter of the node.
    fn param(&self, name: &str) -> Result<&Param<dyn Any>, Error>;
    /// Returns a mutable reference to a named paramter of the node.
    fn param_mut(&mut self, name: &str) -> Result<&mut Param<dyn Any>, Error>;*/
}
