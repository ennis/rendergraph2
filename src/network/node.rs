use crate::error::Error;
use crate::expr::Expr;
use crate::network::{Network, NodeId};
use specs::world::Builder;
use specs::{Component, VecStorage};

pub struct Param {
    expr: Expr,
}

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
}

//--------------------------------------------------------------------------------------------------
pub trait NodeOps {
    /// Returns the name of the node.
    fn name(&self) -> &str;
    /// Returns the display name of the node.
    fn display_node(&self) -> &str {
        self.name()
    }
    /// Returns a reference to a parameter of the node.
    fn param(&self, name: &str) -> Result<&Param, Error>;
    /// Returns a mutable reference to a paramter of the node.
    fn param_mut(&mut self, name: &str) -> Result<&mut Param, Error>;
}
