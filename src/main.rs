use rlua::{Lua, FromLua, ToLua};
use std::fmt::Display;
use std::fmt;
use std::error::Error;

//--------------------------------------------------------------------------------------------------
#[derive(Clone,Debug)]
pub enum EvalError {
    Other,
    TypeError,
}

impl From<rlua::Error> for EvalError {
    fn from(_: rlua::Error) -> Self {
        EvalError::Other
    }
}

impl Display for EvalError {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        match self {
            EvalError::Other => {
                write!(f, "unspecified error")
            }
            EvalError::TypeError => {
                write!(f, "type error")
            }
        }
    }
}

impl Error for EvalError {
}

//--------------------------------------------------------------------------------------------------
pub enum Value {
    Number(f64),
    String(String)
}

impl Value {
    pub fn as_number(&self) -> Result<f64, EvalError> {
        match self {
            Value::Number(v) => Ok(*v),
            _ => Err(EvalError::TypeError)
        }
    }
}

impl From<f64> for Value {
    fn from(v: f64) -> Self {
        Value::Number(v)
    }
}

impl<'lua> From<rlua::Value<'lua>> for Value {
    fn from(v: rlua::Value) -> Self {
        match v {
            rlua::Value::Number(n) => {
                Value::Number(n)
            },
            rlua::Value::String(s) => {
                Value::String(s.to_str().unwrap().to_string())
            }
            _ => unimplemented!()
        }
    }
}

impl<'lua> ToLua<'lua> for Value {
    fn to_lua(self, ctx: rlua::Context<'lua>) -> rlua::Result<rlua::Value<'lua>> {
        match self {
            Value::String(s) => Ok(rlua::Value::String(ctx.create_string(&s)?)),
            Value::Number(num) => Ok(rlua::Value::Number(num)),
            _ => unimplemented!()
        }
    }
}

//--------------------------------------------------------------------------------------------------
pub struct Expr {
    code: String,
}

impl Expr {
    pub fn new(code: impl Into<String>) -> Expr {
        Expr {
            code: code.into()
        }
    }
}
//--------------------------------------------------------------------------------------------------
pub struct ResolutionError;

impl Display for ResolutionError {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(f, "unresolved name")
    }
}
impl Error for ResolutionError {}

pub trait ExprContext {
    /// Finds a variable by name.
    fn var(&self, name: &str) -> Result<Value, ResolutionError>;
}

pub struct FrameContext {
    frame: i64,
    time: f64
}

impl ExprContext for FrameContext {
    fn var(&self, name: &str) -> Result<Value, ResolutionError> {
        match name {
            "frame" => Ok(self.frame.into()),
            "time" => Ok(self.time.into()),
            _ => Err(ResolutionError)
        }
    }
}

//--------------------------------------------------------------------------------------------------
pub struct Evaluator<C: ExprContext> {
    lua: Lua,
    ctx: C,
}

impl <C: ExprContext + Default> Evaluator<C> {
    fn new() -> Evaluator<C> {
        Evaluator {
            lua: Lua::new(),
            ctx: Default::default()
        }
    }
}

impl<C: ExprContext> Evaluator<C> {
    pub fn with_context(ctx: C) -> Evaluator<C> {
        Evaluator {
            lua: Lua::new(),
            ctx
        }
    }

    pub fn ctx(&self) -> &C {
        &self.ctx
    }

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
    fn eval(&mut self, expr: &Expr) -> Result<Value, EvalError> {
        let v : Value = self.lua.context(|ctx| -> Result<Value, rlua::Error> {
            // Somehow, delegate all unresolved names to the context
            // Delegate calls to ch(...) to resolvers
            Ok(ctx.load(&expr.code).eval::<rlua::Value>()?.into())
        })?;
        Ok(v)
    }
}

fn main() {
    let mut ctx = Evaluator::with_context(FrameContext { frame:0, time: 0.0 });
    let pi = Expr::new("3.14159");
    assert_eq!(ctx.eval(&pi).unwrap().as_number().unwrap(), 3.14159);

    // next stop: variables in context
    let time = Expr::new("time * 2");
    assert_eq!(ctx.eval(&time).unwrap().as_number().unwrap(), 2.0);

    // next stop: variable shortcuts


    // next stop: rust closures as expressions

    //println!("Hello, world!");
}
