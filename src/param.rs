use crate::error::Error;
use crate::network::{GenericNodeId, Network, NodeId, Subnet};
use std::any::Any;

pub struct Param<T: Any + Clone> {
    /// Closure called to calculate the value of the parameter
    v: Box<Fn() -> Result<T, Error>>,
    /// Cached value
    cached: Result<T, Error>,
}

impl<T: Any + Clone> Param<T> {
    /// Creates a parameter that just returns a constant.
    pub fn new_constant(
        mut subnet: Subnet,
        name: impl Into<String>,
        v: T,
    ) -> Result<NodeId<Param<T>>, Error> {
        let mut n = subnet.new_node(name);
        let data = Param {
            v: Box::new(move || Ok(v.clone())),
            cached: Err(Error::Other),
        };
        Ok(n.set_data(data))
    }
}
