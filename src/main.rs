#![feature(coerce_unsized, unsize)]
use crate::error::Error;
use crate::eval::{EvalContext, Evaluator};
use crate::expr::{Expr, Value};
use crate::network::node::{NodeOps, Param};
use crate::network::{Network, ParamId};
use std::collections::HashMap;
use std::{fmt, ptr};
use std::fmt::Display;
use std::ops::CoerceUnsized;
use std::any::Any;
use std::marker::{PhantomData, Unsize};
use std::rc::Rc;
use std::cell::RefCell;

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
    //param_1: Param,
    //param_2: Param,
}

impl TestNode {
    pub fn new(name: impl Into<String>) -> TestNode {
        TestNode {
            name: name.into(),
            //param_1: Param::new(Expr::new("0.0")),
            //param_2: Param::new(Expr::new("0.0")),
        }
    }
}

impl NodeOps for TestNode {
    fn name(&self) -> &str {
        "TestNode"
    }

    /*fn param(&self, name: &str) -> Result<&Param, Error> {
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
    }*/
}

//--------------------------------------------------------------------------------------------------


pub struct Param<T>
{
    /// Closure called to calculate the value of the parameter
    v: Box<Fn(&mut dyn EvalContext) -> Result<T, Error>>,
    /// Cached value
    cached: Result<T, Error>,
    /// ID of this parameter in the dependency graph.
    id: ParamId,
}

// e.g.
// A = 1.0
// B = 2.0
// C = A+B
// D = C+2.0
// E = A+B+D
//
// A: {}
// B: {}
// C: {A,B}
// D: {C}
// E: {A,B,D}

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

impl<T> Param<T>
{
    /// Creates a parameter that just returns a constant.
    pub fn constant(v: T) -> Param<T> {
        Param {
            v: Box::new(move || v),
            cached: Err(Error::Other)
        }
    }

    /// Calculates the parameter.
    pub fn eval(&mut self, ctx: &mut dyn EvalContext) -> Result<(), Error> {
        unimplemented!()
    }

    /// Returns the cached value of the parameter.
    pub fn get(&self) -> Result<&T, Error> {
        self.v(ctx)
    }
}


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
    /*n2.param_mut("param_2")
        .unwrap()
        .set_expr(Expr::new("ch('../n1/param_2')"));*/

    // Several things:
    // - setting parameters should always happen within a context
    //   so that dependent parameters can be re-evaluated and re-checked if deemed necessary
    //n1.param_mut("param_1").unwrap().set_expr(Expr::new("1.0"));

    let mut net = Network::new();
    net.open_root().add_named_node("obj");
    net.open_root().add_named_node("render");
    net.open_root().add_named_node("scene");
    net.open_root().add_named_node("present");

    //let n = net.get::<NodeType>("/obj/test1").unwrap(); // -> NodeProxy<NodeType>
    //n.param.bind(n1.param);
    //n.param.bind2(n1.param, n2.param, |n1,n2| { n1 + n2 } )

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

    let par = Par::constant(3.14f32);
    let par2 : &Par<dyn Any> = &par;



    //
    // Cache IDs (or more? pointer addresses?) after lookup.

    // The graph is "just" an ECS.
    // Actual Nodes implement NodeOps.
    // Nodes+Parameter of nodes depend on each other.
    // Parameters are closures that calculate a value from other parameters and context variables.
    //  - possibly by executing a script (lua/wasm)
    // Closures are implemented either in pure rust or in lua, or something else.
    // Closures should be able to tell what their dependencies are.
    //
    // -> declare dependencies, shared param code fetches the values and passes them to the closure (bind1/bind2/bind3 ...)
    //      actual interesting behavior is in the closure

    // 1. Params are just values
    // 2. Params are closures that return values calculated from a context
    // 3. Params are closures that respond to change
    //
}
