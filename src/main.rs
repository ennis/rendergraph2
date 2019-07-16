use crate::error::Error;
use crate::eval::{EvalContext, Evaluator};
use crate::expr::{Expr, Value};
use crate::network::node::NodeOps;
use crate::network::{GenericNodeId, Network, NodeId, Subnet};
use std::any::Any;
use std::marker::PhantomData;
use log::{debug,info,warn,error};
use enum_primitive_derive::Primitive;
use zmq::Message;
use serde::{Serialize, Serializer, Deserialize};

mod error;
mod eval;
mod expr;
mod network;
mod param;

use self::param::Param;

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

#[derive(Clone,Debug,Eq,PartialEq,Deserialize)]
#[serde(tag = "method", content = "data")]
pub enum Method {
    GetVersion{},
    GetNodeInfo{},
    Kill{}
}

#[derive(Copy,Clone,Debug,Eq,PartialEq)]
pub enum NextAction {
    Continue,
    Break,
}

#[derive(Serialize)]
pub struct VersionReply {
    version: u32,
}

#[derive(Clone,Debug,Eq,PartialEq,Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Reply<T> {
    status: i64,
    #[serde(skip_serializing_if="Option::is_none")]
    error_message: Option<String>,
    #[serde(skip_serializing_if="Option::is_none")]
    data: Option<T>,
}

impl<T> Reply<T> {
    pub fn new_error(status: i64, message: String) -> Reply<T> {
        Reply {
            data: None,
            error_message: Some(message),
            status
        }
    }

    pub fn new(data: T) -> Reply<T> {
        Reply {
            data: Some(data),
            error_message: None,
            status: 0
        }
    }
}

pub fn make_reply<T: Serialize>(data: &T) -> Vec<u8> {
    use serde::ser::SerializeStruct;
    let mut out : Vec<u8> = Vec::new();
    let mut s = serde_json::Serializer::new(&mut out);
    let mut state = s.serialize_struct("Reply", 2).unwrap();
    state.serialize_field("status", &0).unwrap();
    state.serialize_field("data", &data).unwrap();
    state.end();
    out
}

pub fn make_error_reply(message: &str) -> Vec<u8> {
    use serde::ser::SerializeStruct;
    let mut out : Vec<u8> = Vec::new();
    let mut s = serde_json::Serializer::new(&mut out);
    let mut state = s.serialize_struct("Reply", 2).unwrap();
    state.serialize_field("status", &1).unwrap();
    state.serialize_field("errorMessage", message).unwrap();
    state.end();
    out
}


pub fn handle_message(net: &mut Network, socket: &zmq::Socket) -> Result<NextAction, String> {
    let msg = socket.recv_msg(0).map_err(|err| err.to_string())?;

    dbg!(msg.as_str().unwrap());

    // deserialize the request
    // this compiles because read_int takes anything that impls FromPrimitive
    let method : Method = serde_json::from_slice(&msg).map_err(|err| format!("invalid method call: {}", err))?;

    let next = match method {
        Method::GetVersion{} => {
            let reply_buf = make_reply(&1i32);
            socket.send(reply_buf, 0);
            NextAction::Continue
        },
        Method::GetNodeInfo{} => {
            unimplemented!()
        },
        Method::Kill{} => {
            NextAction::Break
        }
    };

    Ok(next)
}

const ENDPOINT : &str = "tcp://127.0.0.1:5555";

//--------------------------------------------------------------------------------------------------
fn main() {
    env_logger::init();

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
    let node_b = MyNode::new(net.root_node().subnet());

    // Start zmq server
    let zmq_ctx = zmq::Context::new();
    let socket = zmq_ctx.socket(zmq::SocketType::REP).expect("could not create zmq socket");
    socket.bind(ENDPOINT).expect("could not bind zmq socket");
    info!("Listening on {}", ENDPOINT);

    loop {
        let r = handle_message(&mut net, &socket);
        match r {
            Err(msg) => {
                error!("error processing message: {}", msg);
                panic!()
            }
            Ok(next) => {
                match next {
                    NextAction::Continue => continue,
                    NextAction::Break => break
                }
            }
        }
    }
}
