//--------------------------------------------------------------------------------------------------
use crate::error::Error;
use crate::expr::{Expr, Value};
use rlua::Lua;
use rlua::ToLua;

pub trait EvalContext {
    /// Finds a variable by name.
    fn var(&self, name: &str) -> Result<Value, Error>;
}

//--------------------------------------------------------------------------------------------------
pub struct Evaluator<C: EvalContext> {
    lua: Lua,
    ctx: C,
}

impl<C: EvalContext + Default> Evaluator<C> {
    fn new() -> Evaluator<C> {
        Evaluator {
            lua: Lua::new(),
            ctx: Default::default(),
        }
    }
}

impl<C: EvalContext> Evaluator<C> {
    /// Creates a new evaluator using the given context for resolving variables.
    pub fn with_context(ctx: C) -> Evaluator<C> {
        Evaluator {
            lua: Lua::new(),
            ctx,
        }
    }

    /// Returns a reference to the attached context.
    pub fn ctx(&self) -> &C {
        &self.ctx
    }

    /// Returns a mutable reference to the attached context.
    pub fn ctx_mut(&mut self) -> &mut C {
        &mut self.ctx
    }

    /// Sets the value of a variable in the context, or creates it if it doesn't exist.
    fn set_var(&mut self, var: &str, value: impl Into<Value>) {
        self.lua.context(|ctx| {
            let g = ctx.globals();
            g.set(var, value.into())
        });
    }

    /// Evaluates an expression.
    pub fn eval(&mut self, expr: &Expr) -> Result<Value, Error> {
        let v: Value = self.lua.context(|luactx| -> Result<Value, rlua::Error> {
            luactx.scope(|scope| {
                luactx.globals().set(
                    "v",
                    scope.create_function_mut(|luactx, s: String| {
                        Ok(self
                            .ctx
                            .var(&s)
                            .map(|v| v.to_lua(luactx).unwrap())
                            .unwrap_or(rlua::Value::Nil))
                    })?,
                )?;
                Ok(luactx.load(&expr.code()).eval::<rlua::Value>()?.into())
            })
        })?;
        Ok(v)
    }
}
