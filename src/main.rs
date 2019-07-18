use crate::error::Error;
use crate::eval::{EvalContext, Evaluator};
use crate::expr::{Expr, Value};
use crate::network::node::NodeOps;
use crate::network::{GenericNodeId, Network, NodeId, Subnet};
use enum_primitive_derive::Primitive;
use log::{debug, error, info, warn};
use serde::{Deserialize, Serialize, Serializer};
use std::any::Any;
use std::marker::PhantomData;

mod error;
mod eval;
mod expr;
mod network;
mod param;
mod server;

use self::param::Param;
use crate::server::{
    make_error_reply, make_reply, Request, Server, DEFAULT_ENDPOINT, PROTOCOL_VERSION,
};

#[derive(Clone, Debug)]
pub struct FrameContext {
    frame: i64,
    time: f64,
}

impl EvalContext for FrameContext {
    fn var(&self, name: &str) -> Result<Value, Error> {
        match name {
            "frame" => Ok(self.frame.into()),
            "time" => Ok(self.time.into()),
            _ => Err(Error::NameNotFound),
        }
    }
}

// Who should own the parameter values?
// - the depgraph
//      -> Node stores ParamId<T>
//      -> Poor locality?
// - the node
//      -> node owns closure, cached value
//      -> must 'contact' the node when re-evaluation is needed
//          -> Open node in ECS, call node.param_update(id, ctx)
//          -> No need for closures or wrappers though, which is nice
//      -> node must register params on creation
//
// On node deletion:
// - remove params from depgraph
// - the safer option is to have common logic for removing parameters from the depgraph
//      - implementors cannot get it wrong, and they have nothing to do
//      - implementors register dependencies / automatically removed when node deleted
//              - if the implementor forgets to register a dependency, then simply won't be notified
//
// Creating a node:
// Node::create(network, params) -> NodeId

//--------------------------------------------------------------------------------------------------

struct MyNode {
    param1: NodeId<Param<f32>>,
    param2: NodeId<Param<f32>>,
}

impl MyNode {
    pub fn new(mut subnet: Subnet) -> Result<NodeId<MyNode>, Error> {
        let mut n = subnet.new_node("MyNode"); // borrows subnet (mutably)
        let param1 = Param::new_constant(n.subnet(), "param1", 3.14)?;
        let param2 = Param::new_constant(n.subnet(), "param2", 1.65)?;

        let data = MyNode { param1, param2 };

        Ok(n.set_data(data))
    }
}

//--------------------------------------------------------------------------------------------------

//--------------------------------------------------------------------------------------------------
fn main() {
    env_logger::init();

    /*
    let mut ctx = Evaluator::with_context(FrameContext {
        frame: 0,
        time: 1.0,
    });
    let pi = Expr::new("3.14159");
    assert_eq!(ctx.eval(&pi).unwrap().as_number().unwrap(), 3.14159);

    // next stop: variables in context
    let time = Expr::new("v('time') * 2");
    assert_eq!(ctx.eval(&time).unwrap().as_number().unwrap(), 2.0);

    let mut net = Network::new();
    let node_a = MyNode::new(net.root_node().subnet());
    let node_b = MyNode::new(net.root_node().subnet());*/

    let server = Server::new(DEFAULT_ENDPOINT).expect("could not create server");

    loop {
        server.run(|req| {
            // deserialize the JSON request
            let req: Request = serde_json::from_str(req)?;

            // got a valid request, forward to handler
            let rep = match req {
                Request::GetVersion {} => make_reply(&PROTOCOL_VERSION),
                _ => make_error_reply("unimplemented"),
            };

            Ok(rep)
        });
    }
}
