use crate::error::Error;
use crate::eval::{EvalContext, Evaluator};
use crate::expr::{Expr, Value};
use crate::network::node::{NodeOps, Param};
use crate::network::Network;
use std::collections::HashMap;
use std::fmt;
use std::fmt::Display;

mod error;
mod eval;
mod expr;
mod network;

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

pub struct TestNode {
    name: String,
    param_1: Param,
    param_2: Param,
}

impl TestNode {
    pub fn new(name: impl Into<String>) -> TestNode {
        TestNode {
            name: name.into(),
            param_1: Param::new(Expr::new("0.0")),
            param_2: Param::new(Expr::new("0.0")),
        }
    }
}

impl NodeOps for TestNode {
    fn name(&self) -> &str {
        "TestNode"
    }

    fn param(&self, name: &str) -> Result<&Param, Error> {
        match name {
            "param_1" => Ok(&self.param_1),
            "param_2" => Ok(&self.param_2),
            _ => Err(Error::NameNotFound),
        }
    }

    fn param_mut(&mut self, name: &str) -> Result<&mut Param, Error> {
        match name {
            "param_1" => Ok(&mut self.param_1),
            "param_2" => Ok(&mut self.param_2),
            _ => Err(Error::NameNotFound),
        }
    }
}

//--------------------------------------------------------------------------------------------------

//--------------------------------------------------------------------------------------------------
fn main() {
    let mut ctx = Evaluator::with_context(FrameContext {
        frame: 0,
        time: 1.0,
    });
    let pi = Expr::new("3.14159");
    assert_eq!(ctx.eval(&pi).unwrap().as_number().unwrap(), 3.14159);

    // next stop: variables in context
    let time = Expr::new("v('time') * 2");
    assert_eq!(ctx.eval(&time).unwrap().as_number().unwrap(), 2.0);

    // next stop: nodes
    // - parameters
    // - evaluation stack

    let mut n1 = TestNode::new("n1");
    let mut n2 = TestNode::new("n2");

    // bind n2.param_2 to n1.param_2
    // DESIGN: there must be a way to resolve the expression immediately
    // Modifying a node is indirect, done through a NodeContext that points to the node
    //
    // add nodes into a container (graph)
    // call graph.modify(|ctx| { ... }) to get a context for modification
    // call ctx.set_param(node/id, expr) within closure to set param
    // context keeps track of the mods in a graph
    // -> modify returns the list of validation errors for the whole graph
    //
    //
    n2.param_mut("param_2")
        .unwrap()
        .set_expr(Expr::new("ch('../n1/param_2')"));

    // Several things:
    // - setting parameters should always happen within a context
    //   so that dependent parameters can be re-evaluated and re-checked if deemed necessary
    n1.param_mut("param_1").unwrap().set_expr(Expr::new("1.0"));

    let mut net = Network::new();
    net.open_root().add_named_node("obj");
    net.open_root().add_named_node("render");
    net.open_root().add_named_node("scene");
    net.open_root().add_named_node("present");

    //println!("Hello, world!");

    // networks are stand-alone objects that contain named nodes
    // issue: scope of IDs?
    // - network-local
    // - global (blackboard-local)
    //
    // required relations:
    // - ID -> name
    // - name -> ID
    //
    // Global ID-map of objects
    //
    // - lookup by name
    // - subnet hierarchy (ID -> parent / children)
    //
    // Generational ID

    //
    // Cache IDs (or more? pointer addresses?) after lookup.


}
